//! 插件编写示例
//!
//! 本文件演示如何基于 nonebot_rs 框架编写一个完整的 Plugin。
//! 你可以参照此模板创建自己的插件，放入任意 crate 中。

use nonebot_rs::event::{Event, MessageEvent, NoticeEvent};
use async_trait::async_trait;
use colored::*;
use tracing::{event, Level};

// ============================================================
// 第一步：定义插件的数据结构体
// ============================================================
//
// 每个 Plugin 需要一个结构体来保存插件的状态和配置。
// 结构体需要实现 Debug 和 Clone（Plugin trait 要求 Debug，Clone 用于多线程共享）。
//
// 如果插件不需要状态，可以像 Logger 一样使用空结构体。
// 如果需要可变状态（如计数器），使用 tokio::sync::Mutex 包裹。

/// 示例插件，统计收到各类事件的数量
#[derive(Debug, Clone)]
pub struct PluginExample {
    /// 插件配置：是否启用消息计数
    pub count_messages: bool,
    /// 插件配置：是否启用通知计数
    pub count_notices: bool,
    // 提示：使用 `Option<toml::Value>` 可以从配置文件加载自定义设置
}

// ============================================================
// 第二步：实现内部逻辑
// ============================================================
//
// 将事件处理逻辑写为独立的关联函数或方法。
// 这样做的好处：
//  - 便于单元测试
//  - 保持 Plugin trait 实现简洁
//  - 可以在其他地方复用这些函数

impl PluginExample {

    /// 处理消息事件的业务逻辑
    ///
    /// 接收一个共享计数器和一个消息事件引用，
    /// 根据消息类型（私聊/群聊）记录不同的日志。
    async fn handle_message(
        counter: &tokio::sync::Mutex<usize>,
        event: &MessageEvent,
    ) {
        // 使用异步互斥锁安全地修改共享状态
        let mut count = counter.lock().await;
        *count += 1;

        match event {
            MessageEvent::Private(p) => {
                event!(
                    Level::INFO,
                    "[PluginExample] 第{}条消息: 私聊 来自 {}({}) -> \"{}\"",
                    *count,
                    p.sender.nickname.blue(),
                    p.user_id.green(),
                    p.raw_message,
                );
            }
            MessageEvent::Group(g) => {
                event!(
                    Level::INFO,
                    "[PluginExample] 第{}条消息: 群聊({}) 来自 {}({}) -> \"{}\"",
                    *count,
                    g.group_id.magenta(),
                    g.sender.nickname.blue(),
                    g.user_id.green(),
                    g.raw_message,
                );
            }
        }
    }

    /// 处理通知事件的业务逻辑
    async fn handle_notice(
        counter: &tokio::sync::Mutex<usize>,
        event: &NoticeEvent,
    ) {
        let mut count = counter.lock().await;
        *count += 1;

        // 根据通知子类型做不同处理
        match event {
            NoticeEvent::GroupIncrease(g) => {
                event!(
                    Level::INFO,
                    "[PluginExample] 第{}条通知: 群({}) 新成员加入 {}",
                    *count,
                    g.group_id.magenta(),
                    g.user_id.green(),
                );
            }
            NoticeEvent::GroupDecrease(g) => {
                event!(
                    Level::INFO,
                    "[PluginExample] 第{}条通知: 群({}) 成员离开 {}",
                    *count,
                    g.group_id.magenta(),
                    g.user_id.green(),
                );
            }
            // 其余通知类型可以按需处理
            _ => {
                event!(
                    Level::DEBUG,
                    "[PluginExample] 第{}条通知: {:?}",
                    *count,
                    event,
                );
            }
        }
    }

    /// 事件接收循环
    ///
    /// 这是插件的核心：不断从 EventReceiver 接收广播的事件，
    /// 分发给对应的处理函数。
    ///
    /// `event_receiver` 是一个 broadcast channel 的接收端，
    /// 所有连接到 Nonebot 的 WebSocket 都会向这个 channel 发送事件。
    /// 每个 Plugin 在 run() 中通过 subscribe() 获得自己的接收端。
    ///
    /// 注意：此方法应当被 spawn 到单独的 tokio task 中，
    /// 否则会阻塞 Plugin::run() 的返回，导致 Nonebot 无法继续启动。
    async fn event_recv(
        self,
        mut event_receiver: nonebot_rs::EventReceiver,
    ) {
        // 创建共享计数器（异步互斥锁，因为需要在 .await 之间持有锁）
        let counter = tokio::sync::Mutex::new(0usize);

        // 循环接收事件，channel 关闭时自动退出
        while let Ok(event) = event_receiver.recv().await {
            match &event {
                Event::Message(m) => {
                    if self.count_messages {
                        Self::handle_message(&counter, m).await;
                    }
                }
                Event::Notice(n) => {
                    if self.count_notices {
                        Self::handle_notice(&counter, n).await;
                    }
                }
                Event::Meta(_) => {
                    // 元事件（心跳、生命周期）通常不需要处理，
                    // 但可以在此记录 Bot 在线状态
                }
                Event::Request(_) => {
                    // 请求事件（好友请求、群邀请）在此处理
                }
                Event::Nonebot(nb_event) => {
                    // Nonebot 内部事件（Bot 连接 / 断开）
                    // 可以利用这些事件做初始化和清理
                    match nb_event {
                        nonebot_rs::event::NbEvent::BotConnect { bot } => {
                            event!(
                                Level::INFO,
                                "[PluginExample] Bot {} 已连接",
                                bot.bot_id.red(),
                            );
                        }
                        nonebot_rs::event::NbEvent::BotDisconnect { bot } => {
                            event!(
                                Level::INFO,
                                "[PluginExample] Bot {} 已断开",
                                bot.bot_id.red(),
                            );
                        }
                    }
                }
            }
        }
    }
}

// ============================================================
// 第三步：实现 Plugin trait
// ============================================================
//
// Plugin trait 定义了三个必须实现的方法：
//
// 1. `run()`         — 插件启动入口，在 Nonebot 启动时调用一次。
//                       应该在此订阅 EventReceiver 并 spawn 事件循环。
//                       不能阻塞！长时间运行的逻辑必须 spawn。
//
// 2. `plugin_name()` — 返回插件的唯一名称字符串。
//                       用于日志标识、配置查找和 Plugin 去重。
//                       确保不同的 Plugin 返回不同的名称。
//
// 3. `load_config()` — 从配置文件加载插件配置。
//                       参数 `config: toml::Value` 是从 Nonebotrs.toml
//                       中读取的 `[plugin_name]` 小节内容。
//                       如果不需要配置，留空实现即可。
//
// 使用步骤：
//   在 main.rs 中调用 `nb.add_plugin(YourPlugin);` 即可注册并启动。

#[async_trait]
impl nonebot_rs::Plugin for PluginExample {
    /// 插件启动函数
    ///
    /// 参数说明：
    /// - `event_receiver`: 从 Nonebot 事件广播中 subscribe 的接收端，
    ///                     所有 Bot 收到的 QQ 事件都会通过此通道传递。
    /// - `bot_getter`:     一个 watch channel 接收端，存储着当前所有已连接的 Bot 映射表。
    ///                     通过 `bot_getter.borrow().get(&self_id)` 可以获取特定 Bot 实例，
    ///                     进而调用 Onebot API（如发送消息、获取群列表等）。
    ///
    /// 注意：此函数不应阻塞。请使用 `tokio::spawn` 将事件循环放入后台任务。
    fn run(
        &self,
        event_receiver: nonebot_rs::EventReceiver,
        _bot_getter: nonebot_rs::BotGetter,
    ) {
        // 克隆 self，将插件的所有权移入异步任务
        let plugin = self.clone();
        // 将事件循环 spawn 到 tokio 运行时，避免阻塞 Nonebot 启动
        tokio::spawn(plugin.event_recv(event_receiver));

        event!(
            Level::INFO,
            "{} 插件已启动",
            self.plugin_name().green(),
        );
    }

    /// 返回插件唯一标识名称
    ///
    /// 此名称用于：
    /// - Nonebotrs.toml 中的配置节名 `[plugin_example]`
    /// - 日志中的插件标识
    /// - `nb.remove_plugin("PluginExample")` 移除插件
    ///
    /// 约定：使用 PascalCase，与结构体名称保持一致。
    fn plugin_name(&self) -> &'static str {
        "PluginExample"
    }

    /// 从配置文件加载插件配置
    ///
    /// 配置示例（Nonebotrs.toml）：
    /// ```toml
    /// [plugin_example]
    /// count_messages = true
    /// count_notices = false
    /// ```
    ///
    /// Nonebot 在启动时自动读取配置并调用此方法。
    /// 使用 `config.try_into::<YourConfigStruct>()` 可以反序列化为强类型结构体。
    async fn load_config(&mut self, config: toml::Value) {
        // 从 TOML 配置中提取字段，如果不存在则使用默认值
        if let Some(count_messages) = config
            .get("count_messages")
            .and_then(|v| v.as_bool())
        {
            self.count_messages = count_messages;
        }
        if let Some(count_notices) = config
            .get("count_notices")
            .and_then(|v| v.as_bool())
        {
            self.count_notices = count_notices;
        }
        event!(
            Level::INFO,
            "[{}] 已加载配置: count_messages={}, count_notices={}",
            self.plugin_name().red(),
            self.count_messages,
            self.count_notices,
        );
    }
}

// ============================================================
// 辅助函数：创建 Plugin 实例的工厂方法（可选）
// ============================================================
//
// 提供一个构造函数，方便在 main.rs 中一行创建并注册插件。
// 也方便在创建时传入初始配置值。

impl PluginExample {
    /// 创建带有默认配置的插件实例
    pub fn new() -> Self {
        PluginExample {
            count_messages: true,
            count_notices: true,
        }
    }
}

impl Default for PluginExample {
    fn default() -> Self {
        Self::new()
    }
}
