use crate::matcher::Matcher;
use async_trait::async_trait;
use colored::*;
use nonebot_rs::event::{Event, MessageEvent, MetaEvent, NoticeEvent, RequestEvent, SelfId};
use std::collections::{BTreeMap, HashMap};
use tokio::sync::broadcast;
use tracing::{event, Level};

/// 按 `priority` 依序存储 `MatchersHashMap`
pub type MatchersBTreeMap<E> = BTreeMap<i8, MatchersHashMap<E>>;
/// 使用唯一名字存储 `Matcher`
pub type MatchersHashMap<E> = HashMap<String, Matcher<E>>;
/// Matchers 内部 Action
#[derive(Clone, Debug)]
pub enum MatchersAction {
    /// 添加 MessageEvent Matcher
    AddMessageEventMatcher {
        message_event_matcher: Matcher<nonebot_rs::event::MessageEvent>,
    },
    /// 移除 Matcher
    RemoveMatcher { matcher_name: String },
}

/// Matchers Action Sender
pub type ActionSender = broadcast::Sender<MatchersAction>;

pub const PLUGIN_NAME: &'static str = "Matcher";

/// 根据 `Event` 类型分类存储对应的 `Matcher`
#[derive(Clone, Debug)]
pub struct Matchers {
    /// Event 对应 MatcherBTreeMap，最先匹配，可匹配任意粒度的事件
    pub event: MatchersBTreeMap<Event>,
    /// MessageEvent 对应 MatcherBTreeMap
    pub message: MatchersBTreeMap<MessageEvent>,
    /// NoticeEvent 对应 MatcherBTreeMap
    pub notice: MatchersBTreeMap<NoticeEvent>,
    /// RequestEvent 对应 MatcherBTreeMap
    pub request: MatchersBTreeMap<RequestEvent>,
    /// MetaEvent 对应 MatcherBTreeMap
    pub meta: MatchersBTreeMap<MetaEvent>,
    /// Bot Watch channel Receiver
    bot_getter: Option<nonebot_rs::BotGetter>,
    /// Matchers Action Sender
    action_sender: ActionSender,
    /// Config
    config: HashMap<String, HashMap<String, toml::Value>>,
}

impl Matchers {
    async fn handle_events(&mut self, event: Event, bot: &nonebot_rs::bot::Bot) {
        let message_sent = matches!(event, Event::MessageSent(_));
        // event 层先匹配，block 时不再进入分类 Matcher
        if self
            .handle_event(self.event.clone(), event.clone(), bot.clone(), message_sent)
            .await
        {
            return;
        }
        match event {
            Event::Message(e) => {
                self.handle_event(self.message.clone(), e, bot.clone(), false)
                    .await;
            }
            Event::MessageSent(e) => {
                self.handle_event(self.message.clone(), e, bot.clone(), true)
                    .await;
            }
            Event::Notice(e) => {
                self.handle_event(self.notice.clone(), e, bot.clone(), false)
                    .await;
            }
            Event::Request(e) => {
                self.handle_event(self.request.clone(), e, bot.clone(), false)
                    .await;
            }
            Event::Meta(e) => {
                self.handle_event(self.meta.clone(), e, bot.clone(), false)
                    .await;
            }
            Event::Nonebot(e) => match e {
                nonebot_rs::event::NbEvent::BotConnect { bot } => {
                    log_load_matchers(&self);
                    self.run_on_connect(bot, false).await;
                }
                nonebot_rs::event::NbEvent::BotDisconnect { bot } => {
                    self.run_on_connect(bot, true).await;
                }
            },
        }
    }

    async fn handle_event<E>(
        &mut self,
        matcherb: MatchersBTreeMap<E>,
        event: E,
        bot: nonebot_rs::bot::Bot,
        message_sent: bool,
    ) -> bool
    where
        E: Clone + Send + 'static + std::fmt::Debug + SelfId,
    {
        event!(Level::TRACE, "handling event {:?}", event);
        for (_, matcherh) in matcherb.iter() {
            if self
                ._handler_event(matcherh, event.clone(), bot.clone(), message_sent)
                .await
            {
                return true;
            };
        }
        false
    }

    #[doc(hidden)]
    async fn _handler_event<E>(
        &mut self,
        matcherh: &MatchersHashMap<E>,
        e: E,
        bot: nonebot_rs::bot::Bot,
        message_sent: bool,
    ) -> bool
    where
        E: Clone + Send + 'static + std::fmt::Debug + SelfId,
    {
        event!(Level::TRACE, "handling event_ {:?}", e);
        let mut get_block = false;
        let mut temp_to_remove: Vec<String> = Vec::new();
        let config = bot.config.clone();
        for (name, matcher) in matcherh.iter() {
            let built_matcher = matcher.build(bot.clone());
            if message_sent {
                let handler = built_matcher.get_handler().read().await;
                if !handler.match_message_sent_dyn() {
                    continue;
                }
            }
            let matched = built_matcher
                .match_(e.clone(), config.clone(), self)
                .await;
            if matched {
                if matcher.show_matched_log {
                    event!(Level::INFO, "Matched {}", name.blue());
                }
                if matcher.is_block() {
                    get_block = true;
                }
                if matcher.is_temp() {
                    event!(Level::INFO, "Remove matched temp matcher {}", name.blue());
                    temp_to_remove.push(name.clone());
                }
            }
        }
        for name in temp_to_remove {
            self.remove_matcher(&name);
        }
        get_block
    }

    async fn event_recv(mut self, mut event_receiver: nonebot_rs::EventReceiver) {
        let mut receiver = self.action_sender.subscribe();
        while let Ok(event) = event_receiver.recv().await {
            match receiver.try_recv() {
                Ok(action) => self.handle_action(action),
                Err(_) => {}
            }

            if let Some(bot_getter) = self.bot_getter.clone() {
                let bots = bot_getter.borrow().clone();
                if let Some(bot) = bots.get(&event.get_self_id()) {
                    self.handle_events(event, bot).await;
                }
            }
        }
    }

    /// Matchers 处理 action method
    pub fn handle_action(&mut self, action: MatchersAction) {
        match action {
            MatchersAction::AddMessageEventMatcher {
                message_event_matcher,
            } => {
                event!(
                    Level::DEBUG,
                    "Adding Message Event Matcher: {}",
                    message_event_matcher.name.blue()
                );
                self.add_message_matcher(message_event_matcher);
            }
            MatchersAction::RemoveMatcher { matcher_name } => {
                event!(
                    Level::DEBUG,
                    "Removing Message Event Matcher: {}",
                    matcher_name.blue()
                );
                self.remove_matcher(&matcher_name);
            }
        }
    }
}

#[async_trait]
impl nonebot_rs::Plugin for Matchers {
    fn run(&self, event_receiver: nonebot_rs::EventReceiver, bot_getter: nonebot_rs::BotGetter) {
        let mut m = self.clone();
        m.bot_getter = Some(bot_getter.clone());
        tokio::spawn(m.event_recv(event_receiver));
    }

    fn plugin_name(&self) -> &'static str {
        PLUGIN_NAME
    }

    async fn load_config(&mut self, config: toml::Value) {
        let config: HashMap<String, HashMap<String, toml::Value>> =
            config.try_into().expect("Matchers get error config");
        self.config = config;
        self.load_all_matcher_config().await;
        event!(Level::INFO, "Loaded Matchers config: {:?}", self.config);
    }
}

fn log_load_matchers(matchers: &Matchers) {
    log_matcherb(&matchers.event);
    log_matcherb(&matchers.message);
    log_matcherb(&matchers.notice);
    log_matcherb(&matchers.request);
    log_matcherb(&matchers.meta);
}

fn log_matcherb<E>(matcherb: &MatchersBTreeMap<E>)
where
    E: Clone + Send,
{
    if matcherb.is_empty() {
        return;
    }
    for (_, matcherh) in matcherb {
        for (name, _) in matcherh {
            event!(Level::INFO, "Matcher {} is Loaded", name.blue());
        }
    }
}

impl Matchers {
    /// 新建 Matchers
    pub fn new(
        message: Option<MatchersBTreeMap<MessageEvent>>,
        notice: Option<MatchersBTreeMap<NoticeEvent>>,
        request: Option<MatchersBTreeMap<RequestEvent>>,
        meta: Option<MatchersBTreeMap<MetaEvent>>,
    ) -> Matchers {
        let (sender, _) = broadcast::channel(32);
        Matchers {
            event: BTreeMap::new(),
            message: unoptionb(&message),
            notice: unoptionb(&notice),
            request: unoptionb(&request),
            meta: unoptionb(&meta),
            bot_getter: None,
            action_sender: sender,
            config: HashMap::new(),
        }
    }

    /// 新建空 Matchers
    pub fn new_empty() -> Matchers {
        Matchers::new(None, None, None, None)
    }

    pub fn get(&mut self, m: &Matchers) {
        self.event = m.event.clone();
        self.message = m.message.clone();
        self.notice = m.notice.clone();
        self.request = m.request.clone();
        self.meta = m.meta.clone();
    }

    /// Bot 连接时运行所有 Matcher on_bot_connect 方法
    pub async fn run_on_connect(&self, bot: nonebot_rs::bot::Bot, disconnect: bool) {
        async fn run_on_connect_<E>(
            matcherb: &MatchersBTreeMap<E>,
            bot: nonebot_rs::bot::Bot,
            disconnect: bool,
        ) where
            E: Clone + Send,
        {
            for (_, matcherh) in matcherb {
                for (_, matcher) in matcherh {
                    let built_matcher = matcher.build(bot.clone());
                    let handler = built_matcher.get_handler();
                    let lock_handler = handler.read().await;
                    if disconnect {
                        lock_handler.on_bot_disconnect_dyn(matcher.clone());
                    } else {
                        lock_handler.on_bot_connect_dyn(matcher.clone());
                        lock_handler.init_dyn().await;
                    }
                }
            }
        }

        run_on_connect_(&self.event, bot.clone(), disconnect).await;
        run_on_connect_(&self.message, bot.clone(), disconnect).await;
        run_on_connect_(&self.notice, bot.clone(), disconnect).await;
        run_on_connect_(&self.request, bot.clone(), disconnect).await;
        run_on_connect_(&self.meta, bot.clone(), disconnect).await;
    }

    pub async fn load_all_matcher_config(&self) {
        async fn f<E>(
            matcherb: &MatchersBTreeMap<E>,
            config: &HashMap<String, HashMap<String, toml::Value>>,
        ) where
            E: Clone + Send,
        {
            for (_, matcherh) in matcherb {
                for (matcher_name, matcher) in matcherh {
                    if let Some(data) = config.get(&matcher_name.to_lowercase()) {
                        let handler = matcher.get_handler();
                        let mut lock_handler = handler.write().await;
                        lock_handler.load_config_dyn(data.clone());
                    }
                }
            }
        }

        f(&self.event, &self.config).await;
        f(&self.message, &self.config).await;
        f(&self.notice, &self.config).await;
        f(&self.request, &self.config).await;
        f(&self.meta, &self.config).await;
    }

    #[doc(hidden)]
    fn add_matcher<E>(
        matcherb: &mut MatchersBTreeMap<E>,
        mut matcher: Matcher<E>,
        action_sender: broadcast::Sender<MatchersAction>,
    ) where
        E: Clone + Send,
    {
        matcher.set_action_sender(action_sender);
        match matcherb.get_mut(&matcher.priority) {
            Some(h) => {
                h.insert(matcher.name.clone(), matcher);
            }
            None => {
                let mut hashmap: MatchersHashMap<E> = HashMap::new();
                hashmap.insert(matcher.name.clone(), matcher.clone());
                matcherb.insert(matcher.priority, hashmap);
            }
        }
    }

    /// 向 Matchers 添加 Matcher<Event>
    ///
    /// event 层最先匹配，匹配并 block 时不再进入分类 Matcher
    pub fn add_event_matcher(&mut self, matcher: Matcher<Event>) -> &mut Self {
        Matchers::add_matcher(&mut self.event, matcher, self.action_sender.clone());
        self
    }

    /// 向 Matchers 添加 Matcher<MessageEvent>
    pub fn add_message_matcher(&mut self, matcher: Matcher<MessageEvent>) -> &mut Self {
        Matchers::add_matcher(&mut self.message, matcher, self.action_sender.clone());
        self
    }

    /// 向 Matchers 添加 Vec<Matcher<MessageEvent>>
    pub fn add_message_matchers(&mut self, matchers: Vec<Matcher<MessageEvent>>) -> &mut Self {
        for m in matchers {
            self.add_message_matcher(m);
        }
        self
    }

    /// 向 Matchers 添加 Matcher<NoticeEvent>
    pub fn add_notice_matcher(&mut self, matcher: Matcher<NoticeEvent>) -> &mut Self {
        Matchers::add_matcher(&mut self.notice, matcher, self.action_sender.clone());
        self
    }

    /// 向 Matchers 添加 Matcher<RequestEvent>
    pub fn add_request_matcher(&mut self, matcher: Matcher<RequestEvent>) -> &mut Self {
        Matchers::add_matcher(&mut self.request, matcher, self.action_sender.clone());
        self
    }

    /// 向 Matchers 添加 Matcher<MetaEvent>
    pub fn add_meta_matcher(&mut self, matcher: Matcher<MetaEvent>) -> &mut Self {
        Matchers::add_matcher(&mut self.meta, matcher, self.action_sender.clone());
        self
    }

    /// 根据 Matcher.name 从 Matchers 移除 Matcher
    pub fn remove_matcher(&mut self, name: &str) {
        fn remove_matcher_<E>(matcherb: &mut MatchersBTreeMap<E>, name: &str)
        where
            E: Clone + Send,
        {
            for (_, matcherh) in matcherb.iter_mut() {
                if let Some(_) = matcherh.remove(name) {
                    return;
                }
            }
        }

        remove_matcher_(&mut self.event, name);
        remove_matcher_(&mut self.message, name);
        remove_matcher_(&mut self.notice, name);
        remove_matcher_(&mut self.request, name);
        remove_matcher_(&mut self.meta, name);
    }

    /// 根据 Matcher.name disable Matcher
    pub fn disable_matcher(&mut self, name: &str, disable: bool) {
        fn disable_matcher_<E>(matcherb: &mut MatchersBTreeMap<E>, name: &str, disable: bool)
        where
            E: Clone + Send,
        {
            for (_, matcherh) in matcherb.iter_mut() {
                if let Some(matcher) = matcherh.get_mut(name) {
                    matcher.set_disable(disable);
                }
            }
        }

        disable_matcher_(&mut self.event, name, disable);
        disable_matcher_(&mut self.message, name, disable);
        disable_matcher_(&mut self.notice, name, disable);
        disable_matcher_(&mut self.request, name, disable);
        disable_matcher_(&mut self.meta, name, disable);
    }
}

#[doc(hidden)]
fn unoptionb<K, D>(input: &Option<BTreeMap<K, D>>) -> BTreeMap<K, D>
where
    K: Clone + std::cmp::Ord,
    D: Clone,
{
    match input {
        Some(t) => t.clone(),
        None => BTreeMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::{Handler, Matcher};
    use async_trait::async_trait;
    use nonebot_rs::api_resp::{ApiResp, RespData};
    use nonebot_rs::bot::Bot;
    use nonebot_rs::config::BotConfig;
    use nonebot_rs::event::{EssenceNoticeEvent, GroupMessageEvent, PrivateMessageEvent};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct Counter {
        opt_in: bool,
        count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Handler<MessageEvent> for Counter {
        type Target = MessageEvent;

        fn match_(&self, event: &mut MessageEvent) -> Option<MessageEvent> {
            Some(event.clone())
        }

        fn match_message_sent(&self) -> bool {
            self.opt_in
        }

        async fn handle(&self, _: MessageEvent, _: Matcher<MessageEvent>) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn test_bot() -> Bot {
        let (api_sender, _) = tokio::sync::mpsc::channel::<nonebot_rs::ApiChannelItem>(4);
        let (action_sender, _) = tokio::sync::mpsc::channel::<nonebot_rs::Action>(4);
        let (_, watcher) = tokio::sync::watch::channel(ApiResp {
            status: "ok".to_string(),
            retcode: 0,
            data: RespData::None,
            wording: String::new(),
            message: String::new(),
            echo: String::new(),
        });
        Bot::new(
            "bot".to_string(),
            BotConfig::default(),
            api_sender,
            action_sender,
            watcher,
        )
    }

    fn message_event() -> MessageEvent {
        let json = r#"{"self_id":1,"user_id":2,"time":1,"message_id":1,"message_seq":1,"message_type":"private","sender":{"user_id":2,"nickname":"n"},"raw_message":"hi","font":14,"sub_type":"friend","message":[{"type":"text","data":{"text":"hi"}}],"message_format":"array","raw_pb":"","post_type":"message"}"#;
        match serde_json::from_str::<Event>(json).unwrap() {
            Event::Message(m) => m,
            _ => panic!("expected message event"),
        }
    }

    fn group_message_event() -> MessageEvent {
        let json = r#"{"self_id":1,"user_id":2,"time":1,"message_id":1,"message_seq":1,"message_type":"group","sender":{"user_id":2,"nickname":"n","card":"","role":"member","title":""},"raw_message":"hi","font":14,"sub_type":"normal","message":[{"type":"text","data":{"text":"hi"}}],"message_format":"array","raw_pb":"","post_type":"message","group_id":3,"group_name":"g"}"#;
        match serde_json::from_str::<Event>(json).unwrap() {
            Event::Message(m) => m,
            _ => panic!("expected message event"),
        }
    }

    fn essence_event() -> Event {
        let json = r#"{"time":1789617717,"self_id":1,"post_type":"notice","notice_type":"essence","message_id":-279345864,"sender_id":2,"sub_type":"add","group_id":3,"user_id":2,"operator_id":2}"#;
        serde_json::from_str::<Event>(json).unwrap()
    }

    fn friend_add_event() -> Event {
        let json = r#"{"time":1788602602,"self_id":1,"post_type":"notice","notice_type":"friend_add","user_id":2}"#;
        serde_json::from_str::<Event>(json).unwrap()
    }

    struct EventCounter {
        count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Handler<Event> for EventCounter {
        type Target = Event;

        fn match_(&self, event: &mut Event) -> Option<Event> {
            if matches!(event, Event::Notice(NoticeEvent::Essence(_))) {
                Some(event.clone())
            } else {
                None
            }
        }

        async fn handle(&self, _: Event, _: Matcher<Event>) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct PrivateCounter {
        count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Handler<MessageEvent> for PrivateCounter {
        crate::on_private_message!();

        async fn handle(&self, event: PrivateMessageEvent, _: Matcher<MessageEvent>) {
            assert_eq!(event.user_id, "2");
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct GroupCounter {
        count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Handler<MessageEvent> for GroupCounter {
        crate::on_group_message!();

        async fn handle(&self, event: GroupMessageEvent, _: Matcher<MessageEvent>) {
            assert_eq!(event.group_id, "3");
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    struct EssenceCounter {
        count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Handler<NoticeEvent> for EssenceCounter {
        crate::on_notice!(Essence);

        async fn handle(&self, event: EssenceNoticeEvent, _: Matcher<NoticeEvent>) {
            assert_eq!(event.group_id, "3");
            assert_eq!(event.operator_id, "2");
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn test_event_matcher_fine_grained_and_block() {
        let event_count = Arc::new(AtomicUsize::new(0));
        let message_count = Arc::new(AtomicUsize::new(0));
        let mut matchers = Matchers::new_empty();
        matchers.add_event_matcher(Matcher::new(
            "event",
            EventCounter {
                count: event_count.clone(),
            },
        ));
        matchers.add_message_matcher(Matcher::new(
            "msg",
            Counter {
                opt_in: false,
                count: message_count.clone(),
            },
        ));
        let bot = test_bot();

        matchers.handle_events(essence_event(), &bot).await;
        wait_count(&event_count, 1).await;
        assert_eq!(event_count.load(Ordering::SeqCst), 1);
        assert_eq!(message_count.load(Ordering::SeqCst), 0);

        matchers.handle_events(friend_add_event(), &bot).await;
        matchers
            .handle_events(Event::Message(message_event()), &bot)
            .await;
        wait_count(&message_count, 1).await;
        assert_eq!(event_count.load(Ordering::SeqCst), 1);
        assert_eq!(message_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_typed_notice_handler() {
        let count = Arc::new(AtomicUsize::new(0));
        let mut matchers = Matchers::new_empty();
        matchers.add_notice_matcher(Matcher::new(
            "essence",
            EssenceCounter {
                count: count.clone(),
            },
        ));
        let bot = test_bot();

        matchers.handle_events(essence_event(), &bot).await;
        wait_count(&count, 1).await;
        assert_eq!(count.load(Ordering::SeqCst), 1);

        matchers.handle_events(friend_add_event(), &bot).await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_typed_message_handlers() {
        let private_count = Arc::new(AtomicUsize::new(0));
        let group_count = Arc::new(AtomicUsize::new(0));
        let mut matchers = Matchers::new_empty();
        matchers.add_message_matcher(Matcher::new(
            "private",
            PrivateCounter {
                count: private_count.clone(),
            },
        ));
        matchers.add_message_matcher(Matcher::new(
            "group",
            GroupCounter {
                count: group_count.clone(),
            },
        ));
        let bot = test_bot();

        matchers
            .handle_events(Event::Message(message_event()), &bot)
            .await;
        wait_count(&private_count, 1).await;
        assert_eq!(private_count.load(Ordering::SeqCst), 1);
        assert_eq!(group_count.load(Ordering::SeqCst), 0);

        matchers
            .handle_events(Event::Message(group_message_event()), &bot)
            .await;
        wait_count(&group_count, 1).await;
        assert_eq!(private_count.load(Ordering::SeqCst), 1);
        assert_eq!(group_count.load(Ordering::SeqCst), 1);
    }

    async fn wait_count(count: &AtomicUsize, expected: usize) {
        for _ in 0..500 {
            if count.load(Ordering::SeqCst) >= expected {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        panic!("handler did not run in time");
    }

    #[tokio::test]
    async fn test_message_sent_requires_opt_in() {
        let default_count = Arc::new(AtomicUsize::new(0));
        let opt_in_count = Arc::new(AtomicUsize::new(0));
        let mut matchers = Matchers::new_empty();
        matchers.add_message_matcher(Matcher::new(
            "default",
            Counter {
                opt_in: false,
                count: default_count.clone(),
            },
        ));
        matchers.add_message_matcher(Matcher::new(
            "opt_in",
            Counter {
                opt_in: true,
                count: opt_in_count.clone(),
            },
        ));
        let bot = test_bot();

        matchers
            .handle_events(Event::MessageSent(message_event()), &bot)
            .await;
        wait_count(&opt_in_count, 1).await;
        assert_eq!(default_count.load(Ordering::SeqCst), 0);
        assert_eq!(opt_in_count.load(Ordering::SeqCst), 1);

        matchers
            .handle_events(Event::Message(message_event()), &bot)
            .await;
        wait_count(&default_count, 1).await;
        wait_count(&opt_in_count, 2).await;
        assert_eq!(default_count.load(Ordering::SeqCst), 1);
        assert_eq!(opt_in_count.load(Ordering::SeqCst), 2);
    }
}
