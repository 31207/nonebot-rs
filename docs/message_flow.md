# nonebot-rs WebSocket 消息处理全流程

## 通道清单

所有通道类型定义于 `nonebot_rs/src/lib.rs`：

| 通道 | Rust 类型 | 方向 | 用途 |
|------|-----------|------|------|
| `event_sender` / `event_receiver` | `broadcast::Sender<Event>` / `Receiver<Event>` | WS handler → 所有插件 | 广播 Onebot 事件 |
| `action_sender` / `action_receiver` | `mpsc::Sender<Action>` / `Receiver<Action>` | WS handler/Bot → Nonebot | 增删 Bot、修改配置等内部操作 |
| `api_sender` (per-bot) | `mpsc::Sender<ApiChannelItem>` | 插件 → WS outcome 任务 | 向 Onebot 发送 API 调用 |
| `apiresp_watch_sender` / `api_resp_watcher` (per-bot) | `watch::Sender<ApiResp>` / `Receiver<ApiResp>` | WS income 任务 → Bot | API 调用响应回调 |
| `bot_sender` / `bot_getter` | `watch::Sender<HashMap<String,Bot>>` | Nonebot → 插件 | Bot 增删时通知插件 |
| `shutdown_tx` / `shutdown_rx` | `broadcast::Sender<()>` / `Receiver<()>` | 调用方 → Nonebot | 优雅关闭信号 |
| `matchers_action_sender` | `broadcast::Sender<MatchersAction>` | Matcher → Matchers | 运行时动态注册/移除 Matcher |

## 1. 启动流程

`Nonebot::run()` (`nb.rs:126`)
└─ `async_run()` (`nb.rs:131`)
   ├─ `pre_run()` — 初始化日志、加载插件配置
   ├─ `crate::comms::strat_comms()` — 启动 WS 连接
   └─ `recv()` — 进入 Action 处理循环（直到 shutdown）

### 1.1 WS 连接启动 (`comms.rs:5`)

- **反向 WS (Onebot 连接 nbrs)**: `revs_ws::run()` 绑定 TCP，等待 Onebot 接入
- **正向 WS (nbrs 连接 Onebot)**: `ws::run()` 主动连接 Onebot 的 WS 地址
- 两者均 `tokio::spawn`，与 `recv()` 并发运行

---

## 2. WebSocket 消息接收

### 2.1 正向 WS 连接 (`ws.rs:33`)

1. 构造 HTTP Request（含 Authorization 头）
2. `TcpStream::connect()` 建立 TCP 连接
3. `client_async()` 完成 WS 握手
4. **读取首条消息**，从中提取 `bot_id = event.get_self_id()`
5. 构造本 Bot 的 `mpsc` 和 `watch` 通道
6. 发送 `Action::AddBot { bot_id, api_sender, ... }` 到 `action_sender`
7. 调用 `handler_web_socket()` 进入收发循环

### 2.2 反向 WS 连接 (`revs_ws.rs:41`)

1. TCP 连接到达，检查 `X-Self-ID`、`X-Client-Role`、`Authorization` 头
2. 验证 access_token
3. 升级为 WebSocket
4. 构造通道、发送 `Action::AddBot`
5. 调用 `handler_web_socket()`

### 2.3 收发循环 `handler_web_socket()` (`utils.rs:11`)

```text
┌─────────────────────────────────────────┐
│             WebSocketStream              │
│          socket.split()                  │
│         ┌──────┐    ┌──────┐            │
│         │sink  │    │stream│            │
│         └──┬───┘    └──┬───┘            │
│            │            │                │
│     ┌──────▼──────┐ ┌──▼──────────────┐ │
│     │  outcome    │ │  income          │ │
│     │ (当前任务)  │ │ (tokio::spawn)  │ │
│     │             │ │                 │ │
│     │ 从api_     │ │ stream_recv()   │ │
│     │ receiver   │ │ loop:           │ │
│     │ 读取API     │ │                 │ │
│     │ → JSON     │ │ Event? →       │ │
│     │ → sink.send│ │   broadcast     │ │
│     │             │ │ ApiResp? →     │ │
│     │             │ │   watch.send    │ │
│     │             │ │ Error →        │ │
│     │             │ │   RemoveBot     │ │
│     └─────────────┘ └────────────────┘ │
└─────────────────────────────────────────┘
```

- **income** 被 spawn 为独立 task，持续读取 WS 帧
- **outcome** 在当前 task 中阻塞运行，等待插件发送 API 调用
- income 断开后自行结束，outcome 在 `api_receiver` Sender 被 drop（Bot 被移除）后结束

### 2.4 消息反序列化与分发 `stream_recv()` (`utils.rs:73`)

```
WS帧到达
  │
  ├─ 反序列化 JSON → RecvItem
  │
  ├─ RecvItem::Event(event)
  │   └─ send_event() → broadcast::Sender → 所有插件
  │
  ├─ RecvItem::ApiResp(api_resp)
  │   └─ apiresp_watch_sender.send() → Bot::call_api_resp() 等待者
  │
  └─ JSON 解析失败 → ERROR 日志，继续
  └─ WS 断开 → Action::RemoveBot → 返回 None，income 结束
```

---

## 3. Action 处理（通道中转）

### `Nonebot::recv()` (`nb.rs:100`)

```rust
loop {
    tokio::select! {
        Some(action) = self.action_receiver.recv() => self.handle_action(action),
        _ = self.shutdown_rx.recv() => break,
    }
}
```

### `handle_action()` (`action.rs:27`)

| Action | 行为 |
|--------|------|
| `AddBot` | 注册 Bot 到 `self.bots`，广播 `NbEvent::BotConnect` |
| `RemoveBot` | 移除 Bot，广播 `NbEvent::BotDisconnect` |
| `ChangeBotConfig` | 更新 `bot.config` |

每次注册/移除 Bot 都会通过 `bot_sender.send()` 通知所有 `bot_getter` 订阅者。

---

## 4. 事件路由（broadcast → 插件 → Matcher → Handler）

### 4.1 `Matchers::event_recv()` (`matchers.rs:132`)

```text
event_receiver.recv() ← broadcast channel
  │
  ├─ 非阻塞轮询 MatchersAction（动态增删 Matcher）
  ├─ 从 bot_getter 查找 bot_id 对应的 Bot
  │   └─ 没找到 → 丢弃事件
  └─ handle_events(event, bot)
```

### 4.2 `handle_events()` — 按事件类型分发 (`matchers.rs:49`)

```
Event::Message → handle_event(self.message.clone(), event, bot)
Event::Notice  → handle_event(self.notice.clone(), event, bot)
Event::Request → handle_event(self.request.clone(), event, bot)
Event::Meta    → handle_event(self.meta.clone(), event, bot)
Event::Nonebot → BotConnect → run_on_connect() / BotDisconnect → run_on_disconnect()
```

> **注意**: `handle_events` 会 clone 整个 `BTreeMap`。这是因为 `handle_event` 需要同时持有 `&mut self`（用于移除临时 Matcher）和迭代 Matcher 集合。延迟移除策略（先收集名称、迭代后统一移除）缓解了迭代中修改的问题。

### 4.3 `handle_event()` — 按优先级遍历 (`matchers.rs:77`)

```
遍历 BTreeMap<i8 priority, 升序>
  │  取队头对应 HashMap<String, Matcher<E>>
  ├─ _handler_event() → 遍历该优先级所有 Matcher
  └─ 如果返回 true（block）→ break，低优先级不再处理
```

**优先级机制**: `BTreeMap` 键为 `i8`，负数最小 = 最高优先级。例如 `-1` 优先于 `0` 优先于 `1`。

### 4.4 `_handler_event()` — 逐 Matcher 匹配 (`matchers.rs:97`)

```text
for (name, matcher) in matcherh.iter() {
    调用 matcher.build(bot).match_(event, config, self)
      │
      ├─ matched = true
      │   ├─ 如果 is_block() → get_block = true
      │   └─ 如果 is_temp() → 收集 name 到 temp_to_remove
      │
      └─ matched = false → 继续下一个
}

迭代结束后，统一调用 remove_matcher() 移除 temp_to_remove 中的临时 Matcher
```

---

## 5. Matcher 匹配逻辑 `Matcher::match_()` (`matcher.rs:153`)

```
1. 超时检查
   └─ timestamp > timeout → remove_matcher + timeout_drop → return false

2. 禁用检查
   └─ disable == true → return false

3. 前处理函数组 (pre_matchers)
   └─ 遍历 pre_matchers，逐个调用 fn(&mut E, BotConfig) -> bool
      ├─ to_me() — 私聊始终 true；群聊需 @机器人 或 前缀含昵称
      └─ command_start() — 消息需以命令前缀开头（如 "/"）
   └─ 任一返回 false → return false
   └─ 注意：pre_matcher 可以修改 event（如去掉前缀）

4. 规则组 (rules)
   └─ 遍历 rules，逐个调用 Arc<dyn Fn(&E, &BotConfig) -> bool>
      ├─ is_superuser()
      ├─ is_bot() / is_user()
      ├─ in_group() / in_private_chat()
      ├─ is_private_message_event() / is_group_message_event()
   └─ 任一返回 false → return false

5. Handler 匹配
   └─ handler.match_(&mut event)
      ├─ on_message! → 永远 true
      └─ on_command!("echo") → raw_message 以 "echo" 开头，strip 前缀
   └─ false → return false

6. 匹配成功
   └─ tokio::spawn(handler.handle(event, matcher))
   └─ return true
```

> **关键**: `handler.handle()` 被 spawn 为独立 tokio task，与事件路由并发执行。Matcher 迭代立即继续。

---

## 6. API 调用（插件 → WS → Onebot）

### 6.1 无需响应 (`Bot::call_api()`) (`bot.rs:112`)

```text
Handler 调用
  └─ matcher.send_text("hello")
      └─ Matcher::send(vec![Message::Text(...)])
          └─ Bot::send_by_message_event(event, msg)
              └─ Bot::send_group_msg(group_id, msg)  // 或 send_private_msg
                  └─ 构造 Api::SendGroupMsg { params, echo }
                  └─ api_sender.send(ApiChannelItem::Api(api))
                      ↓
                   outcome task 接收
                      ↓
                   serde_json::to_string(&api)
                      ↓
                   sink.send(TuMessage::text(json_string))
                      ↓
                   WebSocket → Onebot
```

### 6.2 带响应 (`Bot::call_api_resp()`) (`bot.rs:126`)

```text
1. 发送 Api::GetLoginInfo { echo: "GetLoginInfo-1690000000" }
2. 克隆 api_resp_watcher（watch::Receiver）
3. 30秒超时的循环：
   ├─ watcher.changed().await — 等待新响应
   ├─ 检查 watcher.borrow().echo 是否匹配
   │   ├─ 匹配 → return Some(resp)
   │   └─ 不匹配 → 继续等待
   └─ 超时 / 通道关闭 → return None
```

> **注意**: watch channel 只保留最新值。并发 `call_api_resp` 调用者都会看到每次更新，各自通过 echo 判断是否是自己的响应。

---

## 7. 完整数据流图

```
                        Onebot Server
                            │
              ┌─────────────┼─────────────┐
              │ 正向WS        │ 反向WS      │
              │ (nbrs连接)   │ (BBot连接)  │
              ▼              ▼              │
         ws::run()    revs_ws::run()       │
              │              │              │
              └──────┬───────┘              │
                     │                      │
                     ▼                      │
            handler_web_socket()           │
       ┌──────────────┼──────────────┐     │
       │ income        │ outcome      │     │
       │(spawn)        │(当前task)    │     │
       │               │              │     │
       │ WS帧 → JSON  │              │     │
       │    ├─Event    │              │     │
       │    │  └─broadcast │          │     │
       │    │    channel    │         │     │
       │    └─ApiResp  │              │     │
       │       └─watch  │              │     │
       │         channel│              │     │
       │               │ api_receiver │     │
       │               │ ← ApiChannelItem│  │
       │               │ → JSON → WS ─────┘
       └───────────────┴──────────────┘
              │
    ┌─────────┴──────────┐
    │  broadcast::Sender │ ← send_event()
    │     <Event>        │
    └─────────┬──────────┘
              │
    ┌─────────┼──────────────────────┐
    │  Matchers::event_recv()        │  Logger::event_recv()
    │    ├─ 查 Bot (by self_id)      │
    │    └─ handle_events()          │
    │         ├─ handle_event() 按优先级 │
    │         └─ _handler_event() 逐Matcher│
    │              └─ Matcher::match_()  │
    │                   ├─ timeout check │
    │                   ├─ pre_matchers  │
    │                   ├─ rules         │
    │                   └─ handler.match_()
    │                        └─ spawn(handler.handle())
    │                             └─ matcher.send_text()
    │                                  └─ api_sender ─────┐
    └────────────────────────────────────────────────────┼─┘
                                                         │
              ┌──────────────────────────────────────────┘
              ▼
    ┌──────────────────┐
    │   outcome task    │
    │  api_receiver     │
    │  → JSON → WS     │
    └──────────────────┘
```

---

## 8. 内部 Action 流程

```
WS handler                         Nonebot::recv()
     │                                    │
     │  Action::AddBot {                  │
     │    bot_id,                         │
     │    api_sender,                     │
     │    api_resp_watcher                │
     │  }                                 │
     ├──────────────────────────────────► │
     │   action_sender.send()              │
     │                                    ├─ add_bot() → bots.insert()
     │                                    ├─ bot_sender.send() → 通知插件
     │                                    └─ 广播 NbEvent::BotConnect

     │  Action::RemoveBot {               │
     │    bot_id                          │
     │  }                                 │
     ├──────────────────────────────────► │
     │   (WS断开时发送)                    │
     │                                    ├─ remove_bot() → bots.remove()
     │                                    ├─ bot_sender.send() → 通知插件
     │                                    └─ 广播 NbEvent::BotDisconnect
```

---

## 9. 边界情况与错误处理

| 场景 | 文件:行 | 行为 |
|------|---------|------|
| 正向 WS 连接失败 | `ws.rs:59-62` | 返回，外层 loop 等待 5s 后重连 |
| WS 握手失败 | `ws.rs:76-117` | 返回，外层 loop 重连 |
| WS 读取帧为 Err（断开） | `utils.rs:103-109` | 发送 `RemoveBot`，income 任务退出 |
| JSON 反序列化失败 | `utils.rs:93-101` | ERROR 日志，继续读取下一帧 |
| broadcast 通道满 | `utils.rs:115-120` | ERROR 日志，事件被丢弃 |
| 事件 self_id 无对应 Bot | `matchers.rs:141-147` | 静默丢弃 |
| Matcher 超时 | `matcher.rs:164-173` | 移除 Matcher + 调用 timeout_drop() |
| 临时 Matcher 匹配后 | `matchers.rs:120-128` | 收集到 temp_to_remove，迭代后统一移除 |
| block=true 阻隔 | `matchers.rs:90-92` | 低优先级 Matcher 不再处理 |
| API 响应 echo 不匹配 | `bot.rs:148-153` | 忽略，继续等待下一个响应 |
| API 响应超时（30s） | `bot.rs:139-154` | 返回 None |
| API sender 通道关闭 | `bot.rs:128-131` | Ok()? 返回 None |
| ChangeBotConfig bot 不存在 | `action.rs:70` | expect("Bot not found") panic |
| RemoveBot bot 不存在 | `action.rs:60-66` | WARN 日志，不 panic |
