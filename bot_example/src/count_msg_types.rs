use nonebot_rs::Message;
use nonebot_rs::{matcher::prelude::*, message::UniMessage};
pub struct CountMsgType {}

#[async_trait]
impl Handler<MessageEvent> for CountMsgType {
    on_message!(MessageEvent);

    async fn handle(&self, event: MessageEvent, matcher: Matcher<MessageEvent>) {
        let msg = event.get_message();
        let mut text_count = 0;
        let mut image_count = 0;
        let mut other_count = 0;

        for segment in msg.iter() {
            match segment {
                Message::Text(_) => text_count += 1,
                Message::Image(_) => image_count += 1,
                _ => other_count += 1,
            }
        }

        let reply = format!(
            "Message Type Counts:\nText: {}\nImage: {}\nOther: {}",
            text_count, image_count, other_count
        );

        matcher
            .send(UniMessage::new().text(reply.as_str()).build())
            .await;
    }
}

pub fn count_msg_types() -> Matcher<MessageEvent> {
    Matcher::new("count_msg_types", CountMsgType {})
        .add_rule(rules::is_superuser())
        .add_rule(rules::is_private_message_event())
}
