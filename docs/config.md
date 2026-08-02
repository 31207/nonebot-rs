# nonebot-rs 配置系统

## 配置文件：`Nonebotrs.toml`

首次运行时自动生成默认文件（`config.rs:122-138`）。格式如下：

```toml
[global]
debug = true                                          # 开启 debug 日志
trace = false                                         # 可选，开启 trace 日志
event_channel_capacity = 4096                         # 可选，事件通道容量，默认 1024
superusers = ["QQ号码"]                                # 全局超级管理员
nicknames = ["bot昵称"]                                # 机器人昵称（用于 @ 识别）
command_starts = ["/", "#"]                           # 命令前缀

[ws_server]                                           # 反向 WS 服务器（Onebot 连接 nbrs）
host = "0.0.0.0"
port = 8088
access_token = "token"

[bots]                                                # Bot 独立配置
[bots.123456]                                         # bot_id 为节名
superusers = ["789012"]                               # 覆盖全局的超级管理员
nicknames = ["专属昵称"]                                # 覆盖全局的昵称
command_starts = ["!"]                                # 覆盖全局的命令前缀
access_token = "bot_token"                            # 覆盖全局的 access_token
ws_server = "ws://127.0.0.1:6700/ws"                  # 正向 WS 地址

# ═══════ 以下为插件自定义节 ────────

[scheduler]                                           # Scheduler 插件配置
disable = false

[scheduler.jobs.morning]                              # 名为 "morning" 的定时任务
cron = "0 8 * * *"
group_id = "123456"
message = "早上好！"

[scheduler.jobs.reminder]                             # 名为 "reminder" 的定时任务
cron = "0 */2 * * *"
group_id = "123456"
message = "该喝水了！"

[matcher]                                             # Matchers 插件配置
[matcher.echo]                                        # Handler 名为 "echo" 的配置
max_times = 10                                        # 自定义字段

[matcher.rcnb]                                        # Handler 名为 "rcnb" 的配置
custom_option = "value"
```

## 核心数据结构

```rust
// config.rs

pub struct NbConfig {
    pub global: GlobalConfig,                      // [global] 节 → 固定字段
    pub bots: Option<HashMap<String, BotConfig>>,   // [bots.*] 节 → 固定字段
    pub ws_server: Option<WebSocketServerConfig>,    // [ws_server] 节 → 固定字段
    config: Config,                                  // 原始 Config 对象（含所有节）
}

pub struct GlobalConfig {
    pub debug: bool,
    pub trace: Option<bool>,
    pub event_channel_capacity: Option<usize>,       // #[serde(default)]
    pub superusers: Vec<String>,
    pub nicknames: Vec<String>,
    pub command_starts: Vec<String>,
}

pub struct BotConfig {
    pub bot_id: String,                              // #[serde(skip)] 运行时填充
    pub superusers: Vec<String>,                    // #[serde(default)] 空则继承全局
    pub nicknames: Vec<String>,
    pub command_starts: Vec<String>,
    access_token: String,                            // 非 pub，通过 AccessToken 访问
    pub ws_server: String,                           // 非空才启动正向 WS
}
```

## 读取流程

```
Nonebot::new()                               nb.rs:35
  │
  └─ NbConfig::load()                        config.rs:122
       │
       ├─ 文件不存在 ─────────────────────────────────────────────┐
       │   1. NbConfig::default() — 构造默认值                    │
       │   2. toml::to_string(&config) — 序列化为 TOML 文本       │
       │   3. std::fs::write(路径, 文本) — 写入文件               │
       │   4. 控制台输出 "未发现配置文件，已新建配置文件。"        │
       │                                                         │
       └─ 文件存在 ─────────────────────────────────────────────┐ │
           1. config::Config::default()                         │ │
           2. _config.merge(config::File::with_name(...))       │ │
              └─ config crate 读取并解析全部 TOML               │ │
           3. _config.clone().try_into::<NbConfig>()            │ │
              └─ serde 将 [global]/[bots]/[ws_server] 映射到    │ │
                 NbConfig 的对应字段                             │ │
           4. config.config = _config                           │ │
              └─ 保存完整 Config 对象（供插件查询）             │ │
                                                                 │ │
       └─────────────────────────────────────────────────────────┘ │
       └───────────────────────────────────────────────────────────┘

Nonebot::pre_run()                           nb.rs:70-93
  │
  │  for (plugin_name, plugin) in &mut plugins:
  │
  │    // 以插件名(小写)为 key，从 raw Config 查询
  │    let key = plugin.plugin_name().to_lowercase();
  │    let value: Option<toml::Value> = self.config.get_config(&key);
  │                    │
  │                    │  config.rs:141-157
  │                    │  _config.get("scheduler") → 查 [scheduler] 节
  │                    │  _config.get("matcher")   → 查 [matcher] 节
  │                    │  _config.get("myplugin")  → 查 [myplugin] 节
  │                    │
  │    if let Some(cfg) = value {
  │        plugin.load_config(cfg).await;
  │    }
  │
  │    plugin.run(event_receiver, bot_getter);
```

### 关键机制：两层配置

```
TOML 文件 ──► config crate 解析 ──► Config 对象
                                        │
                    ┌───────────────────┴───────────────────┐
                    │                                       │
            已知结构 (serde 自动映射)              原始 Config (插件动态读取)
                    │                                       │
            NbConfig {                              get_config("scheduler")
              global: GlobalConfig,                 get_config("matcher")
              bots: HashMap<...>,                   get_config("自定义插件名")
              ws_server: ...,
            }
```

- **已知结构**：`[global]`、`[bots.*]`、`[ws_server]` 在 `NbConfig` 中有对应字段，serde 自动反序列化
- **插件配置**：TOML 中其余所有节均存储在 `NbConfig.config: Config` 中，由 `get_config(key)` 按插件名查询

## 添加配置

### 方式一：框架级配置

在已有的 struct 中添加字段，自动从对应 TOML 节读取。

**示例**：添加 `log_level` 到 `[global]`

```rust
// config.rs — GlobalConfig
pub struct GlobalConfig {
    pub debug: bool,
    pub trace: Option<bool>,
    #[serde(default)]                          // 使现有文件不报错
    pub log_level: Option<String>,
    // ...其余字段
}

impl Default for NbConfig {
    fn default() -> Self {
        NbConfig {
            global: GlobalConfig {
                // ...
                log_level: None,              // 补充默认值
            },
            // ...
        }
    }
}
```

```toml
# Nonebotrs.toml
[global]
log_level = "warn"
```

使用：`self.config.global.log_level`

### 方式二：插件级配置

定义一个 struct，在 `load_config()` 中从 `toml::Value` 反序列化。

**示例**：自定义 `MyPlugin`

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MyPluginConfig {
    api_key: String,
    enable: bool,
}

struct MyPlugin {
    config: Option<MyPluginConfig>,
}

#[async_trait]
impl Plugin for MyPlugin {
    fn plugin_name(&self) -> &'static str {
        "MyPlugin"          // ← TOML 节名必须匹配 (小写)
    }

    async fn load_config(&mut self, config: toml::Value) {
        self.config = Some(config.try_into().expect("MyPlugin config error"));
        // self.config.api_key, self.config.enable 即可使用
    }

    fn run(&self, _: EventReceiver, _: BotGetter) { /* ... */ }
}
```

```toml
# Nonebotrs.toml — 节名 = plugin_name().to_lowercase()
[myplugin]
api_key = "sk-xxxxxxxx"
enable = true
```

> **注意**：`nb.rs:86` 以 `plugin.plugin_name().to_lowercase()` 为 key 查询。确保 TOML 节名与 Rust 中一致（小写）。

### 方式三：Matcher 级配置

Matchers 插件会将 `[matcher]` 下的配置按 Handler 名称分发。

```toml
# Nonebotrs.toml
[matcher.echo]                    # 匹配 new("echo", ...) 的 Handler
max_times = 10
quit_command = ":quit"

[matcher.rcnb]
encoding_type = "v2"
```

Matchers 的处理流程（`matchers.rs:187-188`）：

```
Matchers::load_config(toml::Value)
  │
  │  config.try_into::<HashMap<String, HashMap<String, toml::Value>>>()
  │  得到: {"echo": {"max_times": 10, ...}, "rcnb": {"encoding_type": "v2"}}
  │
  └─ load_all_matcher_config()
       │
       └─ 遍历所有已注册 Matcher，按 name 匹配
            │
            └─ handler.load_config(inner_hashmap)
```

在 Handler 中接收配置：

```rust
// bot_example/src/echo.rs
#[derive(Clone)]
pub struct Echo {
    max_times: u32,
    quit_command: String,
}

impl Handler<MessageEvent> for Echo {
    fn load_config(&mut self, config: HashMap<String, toml::Value>) {
        if let Some(v) = config.get("max_times").and_then(|v| v.as_integer()) {
            self.max_times = v as u32;
        }
        if let Some(v) = config.get("quit_command").and_then(|v| v.as_str()) {
            self.quit_command = v.to_string();
        }
    }
    // ...
}
```

### 方式四：直接使用任意 TOML 类型

`Plugin::load_config` 接收 `toml::Value`，支持任意 TOML 类型：

```rust
async fn load_config(&mut self, config: toml::Value) {
    // 直接取值
    if let Some(v) = config.get("key").and_then(|v| v.as_str()) { ... }
    if let Some(v) = config.get("num").and_then(|v| v.as_integer()) { ... }

    // 反序列化为 struct
    let cfg: MyConfig = config.try_into().unwrap();

    // 反序列化为 HashMap
    let map: HashMap<String, toml::Value> = config.try_into().unwrap();
}
```

## BotConfig 继承机制

`gen_bot_config()` （`config.rs:165-196`）的合并策略：

```
GlobalConfig 的值                BotConfig 的值                 最终 BotConfig
─────────────────────────────────────────────────────────────────────────────
superusers = ["A"]    +    superusers = ["B"] (非空)    →    superusers = ["B"]
nicknames = ["n1"]    +    nicknames = [] (空)          →    nicknames = ["n1"]
command_starts = ["/"]+    command_starts = [] (空)      →    command_starts = ["/"]
access_token = "t1"   +    access_token = "" (空)        →    access_token = "t1"
access_token = "t1"   +    access_token = "t2" (非空)   →    access_token = "t2"
```

规则：**Bot 配置非空则覆盖，为空则继承全局**。

## 调用时序图

```
main()
 │
 ├─ let mut nb = Nonebot::new()
 │    └─ NbConfig::load()                     // 读文件 + 序列化
 │
 ├─ // 注册插件
 ├─ nb.add_plugin(Scheduler::new())
 ├─ nb.add_plugin(Logger)
 ├─ nb.add_plugin(matchers)
 │
 ├─ nb.run()
 │    └─ async_run()
 │         │
 │         ├─ pre_run()                       // 插件配置加载起点
 │         │    │
 │         │    ├─ log::init()               // 根据 config.global.debug 初始化
 │         │    │
 │         │    ├─ for scheduler:             // plugin_name() = "Scheduler"
 │         │    │    get_config("scheduler")  // → Some(toml::Value)
 │         │    │    load_config(value)       // → try_into::<SchedulerConfig>()
 │         │    │
 │         │    ├─ for matchers:              // plugin_name() = "Matcher"
 │         │    │    get_config("matcher")    // → Some(toml::Value)
 │         │    │    load_config(value)       // → HashMap<name, config>
 │         │    │    │                         // → 分发到各 handler.load_config()
 │         │    │
 │         │    ├─ for logger:                // plugin_name() = "Logger"
 │         │    │    get_config("logger")     // → 查询 [logger] 节
 │         │    │    load_config(value)       // → 空实现，忽略
 │         │    │
 │         │    └─ 各 plugin.run(...)        // 启动插件
 │         │
 │         ├─ strat_comms()                  // 启动 WS 连接
 │         │
 │         └─ recv()                         // Action 处理循环
```

## 注意事项

1. **TOML 节名大小写**：`get_config()` 的 key 来自 `plugin_name().to_lowercase()`，确保 TOML 节名与之匹配
2. **向后兼容**：新增字段使用 `#[serde(default)]` 或 `Option`，避免现有配置文件报错
3. **access_token 的读取**：通过 `gen_access_token()` 生成 `AccessToken` 对象，按 bot_id 查询，支持 `Token xxx` 和 `Bearer xxx` 两种前缀
4. **正向 WS**：Bot 配置中 `ws_server` 非空时才启动正向 WS 连接（`comms.rs:20`）
5. **反向 WS**：`ws_server` 配置存在时启动反向 WS 服务器（`comms.rs:8`）
