use nonebot_rs::config::BotConfig;
use nonebot_rs::event::MessageEvent;
use nonebot_rs::utils::remove_space;
use nonebot_rs::message::{Message,At};
use crate::matcher::PreMatcher;
use std::sync::Arc;

/// 判定消息是否提及 bot（私聊，at，昵称）
pub fn to_me() -> Arc<PreMatcher<MessageEvent>> {
    let to_me = |e: &mut MessageEvent, config: BotConfig| -> bool {
        match e {
            MessageEvent::Private(_) => true,
            MessageEvent::Group(g) => {
                let bot_id = g.self_id.to_string();
                let raw_message = remove_space(&g.raw_message);
                for name in config.nicknames {
                    if raw_message.starts_with(&name) {
                        g.raw_message = remove_space(&raw_message[name.len()..]);
                        return true;
                    }
                }
                for message in &g.message {
                    match message {
                        Message::At( At{ qq: qq_id }) => {
                            if qq_id == &bot_id {
                                g.raw_message = remove_space(
                                    &raw_message.replace(&format!("[CQ:at,qq={}]", g.self_id), ""),
                                );
                                return true;
                            }
                        }
                        _ => continue,
                    }
                }
                false
            }
        }
    };

    Arc::new(to_me)
}

#[doc(hidden)]
fn command_start_(event: &mut MessageEvent, config: BotConfig) -> bool {
    let raw_message = remove_space(&event.get_raw_message());
    let command_starts = config.command_starts;
    if command_starts.is_empty() {
        return true;
    }
    for sc in &command_starts {
        if raw_message.starts_with(sc) {
            let new_raw_message = remove_space(&raw_message[sc.len()..]);
            event.set_raw_message(new_raw_message);
            return true;
        }
    }
    false
}

/// 判定消息是否符合命令起始符
pub fn command_start() -> Arc<PreMatcher<MessageEvent>> {
    Arc::new(command_start_)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nonebot_rs::config::BotConfig;
    use nonebot_rs::event::{GroupMessageEvent, GroupSender, MessageEvent, PrivateMessageEvent, PrivateSender};
    use nonebot_rs::message::{At, Message};

    #[test]
    fn test_to_me_private() {
        let event = MessageEvent::Private(PrivateMessageEvent {
            time: 0,
            self_id: "100".to_string(),
            sub_type: "".to_string(),
            message_id: 1,
            message_seq: 1,
            user_id: "200".to_string(),
            message: vec![],
            message_format: "".to_string(),
            raw_pb: "".to_string(),
            raw_message: "hello".to_string(),
            font: 0,
            sender: PrivateSender {
                user_id: "200".to_string(),
                nickname: "tester".to_string(),
            },
        });

        let mut event = event;
        let config = BotConfig::default();
        let matcher = to_me();
        assert!(matcher(&mut event, config));
    }

    #[test]
    fn test_to_me_group_at_bot() {
        let event = MessageEvent::Group(GroupMessageEvent {
            time: 0,
            self_id: "100".to_string(),
            sub_type: "".to_string(),
            message_id: 1,
            message_seq: 1,
            message_format: "".to_string(),
            raw_pb: "".to_string(),
            group_id: "300".to_string(),
            group_name: "TestGroup".to_string(),
            user_id: "200".to_string(),
            anonymous: None,
            message: vec![Message::At(At { qq: "100".to_string() })],
            raw_message: "[CQ:at,qq=100]".to_string(),
            font: 0,
            sender: GroupSender {
                user_id: "200".to_string(),
                nickname: "tester".to_string(),
                card: "".to_string(),
                role: "member".to_string(),
                title: "".to_string(),
            },
        });

        let mut event = event;
        let config = BotConfig::default();
        let matcher = to_me();
        assert!(matcher(&mut event, config));

        if let MessageEvent::Group(g) = &event {
            assert!(!g.raw_message.contains("[CQ:at,qq=100]"));
        } else {
            panic!("expected Group event");
        }
    }

    #[test]
    fn test_command_start_matches() {
        let event = MessageEvent::Group(GroupMessageEvent {
            time: 0,
            self_id: "100".to_string(),
            sub_type: "".to_string(),
            message_id: 1,
            message_seq: 1,
            message_format: "".to_string(),
            raw_pb: "".to_string(),
            group_id: "300".to_string(),
            group_name: "TestGroup".to_string(),
            user_id: "200".to_string(),
            anonymous: None,
            message: vec![],
            raw_message: "/help".to_string(),
            font: 0,
            sender: GroupSender {
                user_id: "200".to_string(),
                nickname: "tester".to_string(),
                card: "".to_string(),
                role: "member".to_string(),
                title: "".to_string(),
            },
        });

        let mut event = event;
        let mut config = BotConfig::default();
        config.command_starts = vec!["/".to_string()];
        let matcher = command_start();
        assert!(matcher(&mut event, config.clone()));

        if let MessageEvent::Group(g) = &event {
            assert_eq!(g.raw_message, "help");
        } else {
            panic!("expected Group event");
        }
    }
}
