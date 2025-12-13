use crate::config::BotConfig;
use crate::event::MessageEvent;
use crate::event::{SelfId, UserId};
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