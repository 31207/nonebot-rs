/// 注册通配匹配器
///
/// 为 `Matcher` 注册一个匹配所有输入的 `match_` 函数
#[allow(unused_macros)]
#[macro_export]
macro_rules! on_message {
    ($event_type: ty) => {
        type Target = $event_type;
        fn match_(&self, event: &mut $event_type) -> Option<$event_type> {
            Some(event.clone())
        }
    };
}

/// 注册命令匹配器
///
/// 为 `Matcher` 注册一个命令匹配器，匹配的命令将从 `raw_message` 中移除
/// 可以同时接受多个字符串作为同一命令
#[allow(unused_macros)]
#[macro_export]
macro_rules! on_command {
    ($event_type: ty, $command: expr) => {
        type Target = $event_type;
        fn match_(&self, event: &mut $event_type) -> Option<$event_type> {
            if event.get_raw_message().starts_with($command) {
                event.set_raw_message(event.get_raw_message().replace($command, "").to_string());
                Some(event.clone())
            } else {
                None
            }
        }
    };
    ($event_type: ty, $($x:expr),*) => {
        type Target = $event_type;
        fn match_(&self, event: &mut $event_type) -> Option<$event_type> {
            let mut commands:Vec<&str> = Vec::new();
            $(
                commands.push($x);
            )*
            for command in commands.iter() {
                if event.get_raw_message().starts_with(command) {
                    event.set_raw_message(event.get_raw_message().replace(command, "").to_string());
                    return Some(event.clone());
                }
            }
            None
        }
    };
}

/// 注册字符匹配器
///
/// 为 `Matcher` 注册一个字符匹配器，匹配字符将不会移除
/// 可以同时接受多个字符串
#[allow(unused_macros)]
#[macro_export]
macro_rules! on_start_with {
    ($event_type: ty, $command: expr) => {
        type Target = $event_type;
        fn match_(&self, event: &mut $event_type) -> Option<$event_type> {
            if event.get_raw_message().starts_with($command) {
                Some(event.clone())
            } else {
                None
            }
        }
    };
    ($event_type: ty, $($x:expr),*) => {
        type Target = $event_type;
        fn match_(&self, event: &mut $event_type) -> Option<$event_type> {
            let mut commands:Vec<&str> = Vec::new();
            $(
                commands.push($x);
            )*
            for command in commands.iter() {
                if event.get_raw_message().starts_with(command) {
                    return Some(event.clone());
                }
            }
            None
        }
    };
}

/// 注册私聊消息匹配器（类型细化）
///
/// `handle` 将直接收到 `PrivateMessageEvent`
#[allow(unused_macros)]
#[macro_export]
macro_rules! on_private_message {
    () => {
        type Target = $crate::matcher::prelude::PrivateMessageEvent;
        fn match_(
            &self,
            event: &mut $crate::matcher::prelude::MessageEvent,
        ) -> Option<$crate::matcher::prelude::PrivateMessageEvent> {
            match event {
                $crate::matcher::prelude::MessageEvent::Private(e) => Some(e.clone()),
                _ => None,
            }
        }
    };
}

/// 注册群聊消息匹配器（类型细化）
///
/// `handle` 将直接收到 `GroupMessageEvent`
#[allow(unused_macros)]
#[macro_export]
macro_rules! on_group_message {
    () => {
        type Target = $crate::matcher::prelude::GroupMessageEvent;
        fn match_(
            &self,
            event: &mut $crate::matcher::prelude::MessageEvent,
        ) -> Option<$crate::matcher::prelude::GroupMessageEvent> {
            match event {
                $crate::matcher::prelude::MessageEvent::Group(e) => Some(e.clone()),
                _ => None,
            }
        }
    };
}

#[doc(hidden)]
#[allow(unused_macros)]
#[macro_export]
macro_rules! on_notice_variant {
    ($variant:ident, $target:ident) => {
        type Target = $crate::matcher::prelude::$target;
        fn match_(
            &self,
            event: &mut $crate::matcher::prelude::NoticeEvent,
        ) -> Option<$crate::matcher::prelude::$target> {
            match event {
                $crate::matcher::prelude::NoticeEvent::$variant(e) => Some(e.clone()),
                _ => None,
            }
        }
    };
}

/// 注册通知子类型匹配器（类型细化）
///
/// 如 `on_notice!(Essence);`，`handle` 将直接收到 `EssenceNoticeEvent`
/// 具体变体见 `nonebot_rs::event::NoticeEvent`
#[allow(unused_macros)]
#[macro_export]
macro_rules! on_notice {
    (Notify) => {
        $crate::on_notice_variant!(Notify, NotifyNoticeEvent);
    };
    (FriendRecall) => {
        $crate::on_notice_variant!(FriendRecall, FriendRecallNoticeEvent);
    };
    (GroupRecall) => {
        $crate::on_notice_variant!(GroupRecall, GroupRecallNoticeEvent);
    };
    (GroupIncrease) => {
        $crate::on_notice_variant!(GroupIncrease, GroupIncreaseNoticeEvent);
    };
    (GroupDecrease) => {
        $crate::on_notice_variant!(GroupDecrease, GroupDecreaseNoticeEvent);
    };
    (GroupBan) => {
        $crate::on_notice_variant!(GroupBan, GroupBanNoticeEvent);
    };
    (GroupMessageEmojiLike) => {
        $crate::on_notice_variant!(GroupMessageEmojiLike, GroupMessageEmojiLikeNoticeEvent);
    };
    (GroupCard) => {
        $crate::on_notice_variant!(GroupCard, GroupCardNoticeEvent);
    };
    (GroupUpload) => {
        $crate::on_notice_variant!(GroupUpload, GroupUploadNoticeEvent);
    };
    (Essence) => {
        $crate::on_notice_variant!(Essence, EssenceNoticeEvent);
    };
    (FriendAdd) => {
        $crate::on_notice_variant!(FriendAdd, FriendAddNoticeEvent);
    };
    (GroupAdmin) => {
        $crate::on_notice_variant!(GroupAdmin, GroupAdminNoticeEvent);
    };
}

/// 注册任意 Event 匹配器
///
/// 匹配任意 `Event` 模式，如 `on_event!(Event::Notice(NoticeEvent::Essence(_)));`
#[allow(unused_macros)]
#[macro_export]
macro_rules! on_event {
    ($pattern:pat) => {
        type Target = $crate::matcher::prelude::Event;
        fn match_(
            &self,
            event: &mut $crate::matcher::prelude::Event,
        ) -> Option<$crate::matcher::prelude::Event> {
            if matches!(event, $pattern) {
                Some(event.clone())
            } else {
                None
            }
        }
    };
}

#[doc(hidden)]
#[allow(unused_macros)]
#[macro_export]
macro_rules! matcher_request {
    ($b:block) => {
        #[derive(Clone)]
        struct Temp {}

        #[async_trait]
        impl Handler<MessageEvent> for Temp {
            $crate::on_message!(MessageEvent);
            async fn handle(&self, event: MessageEvent, matcher: Matcher<MessageEvent>) {
                $b
            }
        }

        matcher
            .set_message_matcher(
                event.get_self_id(),
                build_temp_message_event_matcher(&event, Temp {}),
            )
            .await;
    };
}
