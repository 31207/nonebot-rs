use nonebot_rs::config::BotConfig;
use nonebot_rs::event::MessageEvent;
use nonebot_rs::event::{NoticeEvent, RequestEvent, SelfId, UserId};
use crate::matcher::Rule;
use std::sync::Arc;

/// 判定 sender 是否为 superuser
pub fn is_superuser<E>() -> Rule<E>
where
    E: UserId,
{
    let is_superuser = |event: &E, config: &BotConfig| -> bool {
        let user_id = event.get_user_id();
        for superuser in &config.superusers {
            if &user_id == superuser {
                return true;
            }
        }
        false
    };
    Arc::new(is_superuser)
}

/// 判定是否为指定 Bot
pub fn is_bot<E>(bot_id: String) -> Rule<E>
where
    E: SelfId,
{
    let is_bot = move |event: &E, _: &BotConfig| -> bool {
        let self_id = event.get_self_id();
        if bot_id == self_id {
            return true;
        }
        false
    };
    Arc::new(is_bot)
}

/// 判定 sender 是否为指定 user
pub fn is_user<E>(user_id: String) -> Rule<E>
where
    E: UserId,
{
    let is_user = move |event: &E, _: &BotConfig| -> bool {
        let id = event.get_user_id();
        if id == user_id {
            return true;
        }
        false
    };
    Arc::new(is_user)
}

/// 判定 event 是否来自指定 group
pub fn in_group(group_id: String) -> Rule<MessageEvent> {
    let in_group = move |event: &MessageEvent, _: &BotConfig| -> bool {
        if let MessageEvent::Group(g) = event {
            if g.group_id == group_id {
                return true;
            }
        }
        false
    };
    Arc::new(in_group)
}

/// 判定 event 是否来自指定 private chat
pub fn in_private_chat(user_id: String) -> Rule<MessageEvent> {
    let in_private_chat = move |event: &MessageEvent, _: &BotConfig| -> bool {
        if let MessageEvent::Private(p) = event {
            if p.user_id == user_id {
                return true;
            }
        }
        false
    };
    Arc::new(in_private_chat)
}

/// 判定 event 是否来自指定groups
pub fn in_groups(group_ids: Vec<String>) -> Rule<MessageEvent> {
    let in_groups = move |event: &MessageEvent, _: &BotConfig| -> bool {
        if let MessageEvent::Group(g) = event {
            if group_ids.contains(&g.group_id) {
                return true;
            }
        }
        false
    };
    Arc::new(in_groups)
}

/// 判定 event 是否为私聊消息事件
pub fn is_private_message_event() -> Rule<MessageEvent> {
    let is_private_message_event = |event: &MessageEvent, _: &BotConfig| -> bool {
        match event {
            MessageEvent::Private(_) => true,
            _ => false,
        }
    };
    Arc::new(is_private_message_event)
}

/// 判定 event 是否为群消息事件
pub fn is_group_message_event() -> Rule<MessageEvent> {
    let is_group_message_event = |event: &MessageEvent, _: &BotConfig| -> bool {
        match event {
            MessageEvent::Group(_) => true,
            _ => false,
        }
    };
    Arc::new(is_group_message_event)
}

/// 判定 notice 是否为指定 notice_type（如 "essence"、"group_admin"）
pub fn is_notice_type(notice_type: &str) -> Rule<NoticeEvent> {
    let notice_type = notice_type.to_string();
    let is_notice_type = move |event: &NoticeEvent, _: &BotConfig| -> bool {
        event.get_notice_type() == notice_type
    };
    Arc::new(is_notice_type)
}

/// 判定 request 是否为指定 request_type（如 "friend"、"group"）
pub fn is_request_type(request_type: &str) -> Rule<RequestEvent> {
    let request_type = request_type.to_string();
    let is_request_type = move |event: &RequestEvent, _: &BotConfig| -> bool {
        event.request_type == request_type
    };
    Arc::new(is_request_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nonebot_rs::event::{PrivateMessageEvent, PrivateSender, UserId};

    struct MockUser {
        user_id: String,
    }

    impl UserId for MockUser {
        fn get_user_id(&self) -> String {
            self.user_id.clone()
        }
    }

    #[test]
    fn test_is_superuser() {
        let mock = MockUser {
            user_id: "123".to_string(),
        };
        let rule = is_superuser::<MockUser>();

        let mut config = BotConfig::default();
        config.superusers = vec!["123".to_string()];
        assert!(rule(&mock, &config));

        config.superusers = vec!["456".to_string()];
        assert!(!rule(&mock, &config));
    }

    #[test]
    fn test_is_notice_type() {
        let json = r#"{"time":1,"self_id":1,"post_type":"notice","notice_type":"essence","message_id":1,"sender_id":2,"sub_type":"add","group_id":3,"user_id":2,"operator_id":2}"#;
        let event = match serde_json::from_str::<nonebot_rs::event::Event>(json).unwrap() {
            nonebot_rs::event::Event::Notice(n) => n,
            _ => panic!("expected notice event"),
        };
        assert!(is_notice_type("essence")(&event, &BotConfig::default()));
        assert!(!is_notice_type("friend_add")(&event, &BotConfig::default()));
    }

    #[test]
    fn test_is_request_type() {
        let event = RequestEvent {
            time: 0,
            self_id: String::new(),
            request_type: "friend".to_string(),
            user_id: String::new(),
            comment: String::new(),
            flag: String::new(),
            sub_type: None,
            group_id: None,
        };
        assert!(is_request_type("friend")(&event, &BotConfig::default()));
        assert!(!is_request_type("group")(&event, &BotConfig::default()));
    }

    #[test]
    fn test_is_private_message_event() {
        let rule = is_private_message_event();
        let event = MessageEvent::Private(PrivateMessageEvent {
            time: 0,
            self_id: String::new(),
            sub_type: String::new(),
            message_id: 0,
            message_seq: 0,
            user_id: String::new(),
            message: vec![],
            message_format: String::new(),
            raw_pb: String::new(),
            raw_message: String::new(),
            font: 0,
            sender: PrivateSender {
                user_id: String::new(),
                nickname: String::new(),
            },
        });
        assert!(rule(&event, &BotConfig::default()));
    }
}