use nonebot_rs::Message;
use nonebot_rs::{matcher::prelude::*, message::UniMessage};
use tracing::event;
pub struct CountMsgType {}

#[async_trait]
impl Handler<MessageEvent> for CountMsgType {
    on_message!(MessageEvent);

    async fn handle(&self, event: MessageEvent, matcher: Matcher<MessageEvent>) {
        let msg = event.get_message();
        let mut text_count = 0;
        let mut image_count = 0;
        let mut at_count = 0;
        let mut other_count = 0;

        for segment in msg.iter() {
            match segment {
                Message::Text(_) => text_count += 1,
                Message::Image(_) => image_count += 1,
                Message::At(_) => at_count += 1,
                Message::Reply(r) => {
                    let id = r.id.parse::<i32>().unwrap();
                    let replied_msg = matcher.get_msg(id).await;
                    println!("Replied msg: {:?}", replied_msg);
                    if let Some(replied_msg) = replied_msg {
                        match replied_msg {
                            nonebot_rs::api_resp::RespMessage::Group(g) => {
                                let replied_msg_content = g.message;
                                for seg in replied_msg_content.iter() {
                                    match seg {
                                        Message::Text(_) => text_count += 1,
                                        Message::Image(_) => image_count += 1,
                                        Message::At(_) => at_count += 1,
                                        _ => other_count += 1,
                                    }
                                }
                            }
                            nonebot_rs::api_resp::RespMessage::Private(p) => {
                                let replied_msg_content = p.message;
                                for seg in replied_msg_content.iter() {
                                    match seg {
                                        Message::Text(_) => text_count += 1,
                                        Message::Image(_) => image_count += 1,
                                        Message::At(_) => at_count += 1,
                                        _ => other_count += 1,
                                    }
                                }
                            }
                        }
                    }
                }
                _ => other_count += 1,
            }
        }

        let reply = format!(
            "Message Type Counts:\nText: {}\nImage: {}\nAt: {}\nOther: {}",
            text_count, image_count, at_count, other_count
        );

        matcher
            .send(UniMessage::new().text(reply.as_str()).build())
            .await;
    }
}

pub fn count_msg_types() -> Matcher<MessageEvent> {
    Matcher::new("count_msg_types", CountMsgType {})
        .add_rule(rules::is_superuser())
        .add_rule(rules::is_group_message_event())
}