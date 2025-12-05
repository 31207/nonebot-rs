use crate::event::{Event, MessageEvent, MetaEvent, NoticeEvent};
use async_trait::async_trait;
use colored::*;
use tracing::{event, Level};

/// Message Event Logger
pub fn message_logger(event: &MessageEvent) {
    match &event {
        MessageEvent::Private(p) => {
            let mut user_id = p.user_id.to_string();
            while user_id.len() < 10 {
                user_id.insert(0, ' ');
            }
            event!(
                Level::INFO,
                "{} [{}] -> {} from {}({})",
                user_id.green(),
                p.self_id.red(),
                p.raw_message,
                p.sender.nickname.blue(),
                p.user_id.green(),
            )
        }
        MessageEvent::Group(g) => {
            let mut group_id = g.group_id.to_string();
            while group_id.len() < 10 {
                group_id.insert(0, ' ');
            }
            event!(
                Level::INFO,
                "{} [{}] -> {} from {}({})[{}]",
                group_id.magenta(),
                g.self_id.red(),
                g.raw_message,
                g.sender.nickname.blue(),
                g.sender.card.yellow(),
                g.user_id.green(),
            )
        }
    }
}

/// Meta Event Logger
pub fn meta_logger(event: &MetaEvent) {
    if &event.meta_event_type == "heartbeat" {
        event!(Level::TRACE, "Recive HeartBeat")
    }
}

pub fn notice_logger(event: &NoticeEvent) {
    match event {
        NoticeEvent::Notify(n) => {
            if let Some(group_id) = &n.group_id {
                event!(
                    Level::INFO,
                    "{} [{}] -> {} 戳了戳 {}",
                    group_id.magenta(),
                    n.self_id.red(),
                    n.user_id.green(),
                    n.target_id.blue(),
                );
            } else {
                event!(
                    Level::INFO,
                    "[{}] -> {} 戳了戳你",
                    n.self_id.red(),
                    n.user_id.green(),
                );
            }
        }
        NoticeEvent::FriendRecall(f) => {
            event!(
                Level::INFO,
                "[{}] -> {}撤回了一条消息",
                f.self_id.red(),
                f.user_id.green(),
            );
        }
        NoticeEvent::GroupRecall(g) => {
            event!(
                Level::INFO,
                "{} [{}] -> {}撤回了一条消息",
                g.group_id.magenta(),
                g.self_id.red(),
                g.user_id.green(),
            );
        }
        NoticeEvent::GroupBan(g) => {
            println!("{:?}", g);
            match g.sub_type.as_str() {
                "ban" => {
                    event!(
                        Level::INFO,
                        "{} [{}] -> {}被{}禁言{}秒",
                        g.group_id.magenta(),
                        g.self_id.red(),
                        g.user_id.green(),
                        g.operator_id.yellow(),
                        g.duration.to_string().red(),
                    );
                }
                _ => match g.duration {
                    0 if g.user_id == "0" => {
                        event!(
                            Level::INFO,
                            "{} [{}] -> 群聊{}被{}解除全员禁言",
                            g.group_id.magenta(),
                            g.self_id.red(),
                            g.group_id.green(),
                            g.operator_id.yellow(),
                        );
                        return;
                    }
                    -1 => {
                        event!(
                            Level::INFO,
                            "{} [{}] -> 群聊{}被{}设置全员禁言",
                            g.group_id.magenta(),
                            g.self_id.red(),
                            g.group_id.green(),
                            g.operator_id.yellow(),
                        );
                        return;
                    }
                    _ => {
                        event!(
                            Level::INFO,
                            "{} [{}] -> {}被{}解除禁言",
                            g.group_id.magenta(),
                            g.self_id.red(),
                            g.user_id.green(),
                            g.operator_id.yellow(),
                        );
                    }
                },
            }
        }
        NoticeEvent::GroupIncrease(g) => {
            event!(
                Level::INFO,
                "{} [{}] -> 群聊人数增加{} 操作者{} 操作类型{}",
                g.group_id.magenta(),
                g.self_id.red(),
                g.user_id.green(),
                g.operator_id.yellow(),
                g.sub_type.blue(),
            );
        }
        NoticeEvent::GroupDecrease(g) => {
            event!(
                Level::INFO,
                "{} [{}] -> 群聊人数减少{} 操作者{} 操作类型{}",
                g.group_id.magenta(),
                g.self_id.red(),
                g.user_id.green(),
                g.operator_id.yellow(),
                g.sub_type.blue(),
            );
        }
        NoticeEvent::GroupMessageEmojiLike(g) => {
            if g.is_add == true {
                event!(
                    Level::INFO,
                    "{} [{}] -> {} 添加了一个emoji({}) to msg({})",
                    g.group_id.magenta(),
                    g.self_id.red(),
                    g.user_id.green(),
                    g.likes.to_string().yellow(),
                    g.message_id.to_string().blue(),
                );
            } else {
                event!(
                    Level::INFO,
                    "{} [{}] -> {} 删除了一个emoji({}) at msg({})",
                    g.group_id.magenta(),
                    g.self_id.red(),
                    g.user_id.green(),
                    g.likes.to_string().yellow(),
                    g.message_id.to_string().blue(),
                );
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Logger;

impl Logger {
    async fn event_recv(self, mut event_receiver: crate::EventReceiver) {
        while let Ok(event) = event_receiver.recv().await {
            match &event {
                Event::Message(m) => message_logger(m),
                Event::Notice(m) => notice_logger(m),
                Event::Meta(m) => meta_logger(m),
                _ => {}
            }
        }
    }
}

#[async_trait]
impl crate::Plugin for Logger {
    fn run(&self, event_receiver: crate::EventReceiver, _: crate::BotGetter) {
        let l = self.clone();
        tokio::spawn(l.event_recv(event_receiver));
    }

    fn plugin_name(&self) -> &'static str {
        "Logger"
    }

    async fn load_config(&mut self, _: toml::Value) {}
}
