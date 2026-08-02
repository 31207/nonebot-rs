use nonebot_rs::config::BotConfig;
use nonebot_rs::event::{MessageEvent, SelfId};
use nonebot_rs::utils::timestamp;
use nonebot_rs::Action;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod matchers;
#[doc(hidden)]
pub mod event_matcher;
/// Preludo for Matcher
pub mod prelude;
pub mod rules;
pub mod prematchers;
pub mod macros;


/// rule 函数类型
pub type Rule<E> = Arc<dyn Fn(&E, &BotConfig) -> bool + Send + Sync>;
/// permatcher 函数类型
pub type PreMatcher<E> = fn(&mut E, BotConfig) -> bool;

/// 单个匹配器，参与匹配的最小单元
///
/// Matcher 匹配器，每个匹配器对应一个 handle 函数
#[derive(Clone)]
pub struct Matcher<E>
where
    E: Clone,
{
    /// Matcher 名称，是 Matcher 的唯一性标识
    pub name: String,
    /// Bot
    pub bot: Option<nonebot_rs::bot::Bot>,
    /// Matchers Action Sender
    action_sender: Option<matchers::ActionSender>,
    /// Matcher 的匹配优先级
    pub priority: i8,
    /// 前处理函数组，获取 &mut event
    pre_matchers: Vec<Arc<PreMatcher<E>>>,
    /// rule 组
    rules: Vec<Rule<E>>,
    /// 是否阻止事件向下一级传递
    pub block: bool,
    /// Matcher 接口函数与可配置项结构体
    handler: Arc<RwLock<dyn Handler<E> + Sync + Send>>,
    /// 是否被禁用
    pub disable: bool,
    /// 是否为临时 Matcher
    pub temp: bool,
    /// 过期时间戳
    pub timeout: Option<i64>,

    #[doc(hidden)]
    event: Option<E>,
}

#[doc(hidden)]
impl<E> std::fmt::Debug for Matcher<E>
where
    E: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Matcher")
            .field("name", &self.name)
            .field("priority", &self.priority)
            .field("block", &self.block)
            .field("disable", &self.disable)
            .field("temp", &self.temp)
            .field("timeout", &self.timeout)
            .field("bot", &self.bot)
            .finish()
    }
}

/// Matcher 接口 trait
#[async_trait]
pub trait Handler<E>
where
    E: Clone,
{
    /// 新 Bot 连接时，调用该函数
    fn on_bot_connect(&self, _: Matcher<E>) {}
    /// Bot 断开连接时，调用该函数
    fn on_bot_disconnect(&self, _: Matcher<E>) {}
    /// timeout drop 函数
    fn timeout_drop(&self, _: &Matcher<E>) {}
    /// 匹配函数
    fn match_(&self, event: &mut E) -> bool;
    /// 处理函数
    async fn handle(&self, event: E, matcher: Matcher<E>);
    /// Load config
    #[allow(unused_variables)]
    fn load_config(&mut self, config: HashMap<String, toml::Value>) {}
    /// 方法初始化函数
    async fn init(&self) {}
}

impl<E> Matcher<E>
where
    E: Clone,
{
    pub fn new<H>(name: &str, handler: H) -> Matcher<E>
    where
        H: Handler<E> + Sync + Send + 'static,
    {
        // 默认 Matcher
        Matcher {
            name: name.to_string(),
            bot: None,
            action_sender: None,
            priority: 1,
            pre_matchers: vec![],
            rules: vec![],
            block: true,
            handler: Arc::new(RwLock::new(handler)),
            disable: false,
            temp: false,
            timeout: None,

            event: None,
        }
    }

    #[doc(hidden)]
    fn pre_matcher_handle(&self, event: &mut E, config: BotConfig) -> bool {
        // 遍历 pre_matcher 处理
        for premather in &self.pre_matchers {
            if !premather(event, config.clone()) {
                return false;
            }
        }
        true
    }

    #[doc(hidden)]
    fn check_rules(&self, event: &E, config: &BotConfig) -> bool {
        // 一次性检查当前事件是否满足所有 Rule
        // check the event fit all the rules or not
        for rule in &self.rules {
            if !rule(event, config) {
                return false;
            }
        }
        true
    }

    #[doc(hidden)]
    pub async fn match_(
        &self,
        event: E,
        config: BotConfig,
        matchers: &mut matchers::Matchers,
    ) -> bool
    where
        E: Send + 'static + SelfId,
    {
        // Matcher 处理流程，匹配成功返回 true 并行处理 handler
        let mut event = event.clone();
        if let Some(timeout) = self.timeout {
            if timestamp() > timeout {
                matchers.remove_matcher(&self.name);
                {
                    let handler = self.handler.read().await;
                    handler.timeout_drop(&self);
                }
                return false;
            }
        }
        if self.disable {
            return false;
        }
        if !self.pre_matcher_handle(&mut event, config.clone()) {
            return false;
        }
        if !self.check_rules(&event, &config) {
            return false;
        }
        {
            let handler = self.handler.read().await;
            if !handler.match_(&mut event) {
                return false;
            }
            let matcher = self.clone().set_event(&event);
            let handler = self.handler.clone();
            tokio::spawn(async move {
                let handler = handler.read().await;
                handler.handle(event, matcher).await
            });
        }
        return true;
    }

    /// 发送 nbrs 内部设置 Action
    pub async fn set(&self, set: Action) {
        if let Some(bot) = &self.bot {
            bot.action_sender.send(set).await.ok();
        }
    }

    /// 向 Matchers 添加 Matcher<MessageEvent>
    pub async fn set_message_matcher(&self, matcher: Matcher<MessageEvent>) {
        let action = matchers::MatchersAction::AddMessageEventMatcher {
            message_event_matcher: matcher,
        };
        if let Some(action_sender) = &self.action_sender {
            let _ = action_sender.send(action);
        } else {
            tracing::event!(tracing::Level::WARN, "Action Sender not init.")
        }
    }

    /// 设置 Matcher 的 bot
    ///
    /// 当前 Matcher 如果已经预设 Bot 将会忽视传入的 Bot
    pub fn build(&self, bot: nonebot_rs::bot::Bot) -> Matcher<E> {
        let mut m = self.clone();
        if let None = &m.bot {
            m.bot = Some(bot);
        }
        m
    }

    /// 为 Matcher 添加向 Matchers 发送 Matchers Action 的 Sender
    /// 会在向 Matchers 添加时调用
    pub fn set_action_sender(&mut self, action_sender: matchers::ActionSender) {
        self.action_sender = Some(action_sender);
    }

    /// 设置 priority
    pub fn set_priority(&mut self, priority: i8) -> Matcher<E> {
        self.priority = priority;
        self.clone()
    }

    /// 添加 pre_matcher 函数
    pub fn add_pre_matcher(&mut self, pre_matcher: Arc<PreMatcher<E>>) -> Matcher<E> {
        self.pre_matchers.push(pre_matcher);
        self.clone()
    }

    /// 添加 rule 函数
    pub fn add_rule(&mut self, rule: Rule<E>) -> Matcher<E> {
        self.rules.push(rule);
        self.clone()
    }

    /// 设置是否阻塞消息向下一级 priority 传递
    pub fn set_block(&mut self, block: bool) -> Matcher<E> {
        self.block = block;
        self.clone()
    }

    /// 获取 handler
    pub fn get_handler(&self) -> &Arc<RwLock<dyn Handler<E> + Sync + Send>> {
        &self.handler
    }

    /// 设置 handler
    pub fn set_handler(
        &mut self,
        handler: Arc<RwLock<dyn Handler<E> + Sync + Send>>,
    ) -> Matcher<E> {
        self.handler = handler;
        self.clone()
    }

    /// 设置是否 disable
    pub fn set_disable(&mut self, disable: bool) -> Matcher<E> {
        self.disable = disable;
        self.clone()
    }

    #[doc(hidden)]
    pub fn set_event(&mut self, event: &E) -> Matcher<E> {
        self.event = Some(event.clone());
        self.clone()
    }

    /// 返回 bolck
    pub fn is_block(&self) -> bool {
        self.block
    }

    /// 判定是否为临时 Matcher
    pub fn is_temp(&self) -> bool {
        self.temp
    }

    /// 设置是否为临时 Matcher
    pub fn set_temp(&mut self, temp: bool) -> Matcher<E> {
        self.temp = temp;
        self.clone()
    }

    /// 设置 Matcher 超时时限
    pub fn set_timeout(&mut self, timeout: i64) -> Matcher<E> {
        self.timeout = Some(timeout);
        self.clone()
    }
}

/// 构建 timeout 为 30s 的临时 Matcher<MessageEvent>
pub fn build_temp_message_event_matcher<H>(
    event: &MessageEvent,
    handler: H,
) -> Matcher<MessageEvent>
where
    H: Handler<MessageEvent> + Send + Sync + 'static,
{
    use nonebot_rs::event::UserId;
    let mut m = Matcher::new(
        &format!(
            "{}-{}-{}",
            event.get_self_id(),
            event.get_user_id(),
            event.get_time()
        ),
        handler,
    )
    .add_rule(crate::matcher::rules::is_user(event.get_user_id()))
    .add_rule(crate::matcher::rules::is_bot(event.get_self_id()));
    if let MessageEvent::Group(g) = event {
        m.add_rule(crate::matcher::rules::in_group(g.group_id.clone()));
    } else {
        m.add_rule(crate::matcher::rules::is_private_message_event());
    }
    m.set_priority(0)
        .set_temp(true)
        .set_timeout(timestamp() + 30)
}
