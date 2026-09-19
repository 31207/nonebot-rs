use builtin_plugins::matcher::prelude::*;
use nonebot_rs::api_resp::RespMessage;
use nonebot_rs::message::UniMessage;

#[derive(Default)]
struct Counts {
    text: usize,
    image: usize,
    at: usize,
    other: usize,
}

impl Counts {
    fn add(&mut self, segment: &Message) {
        match segment {
            Message::Text(_) => self.text += 1,
            Message::Image(_) => self.image += 1,
            Message::At(_) => self.at += 1,
            _ => self.other += 1,
        }
    }

    fn add_all(&mut self, segments: &[Message]) {
        for segment in segments {
            self.add(segment);
        }
    }
}

impl std::fmt::Display for Counts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Message Type Counts:\nText: {}\nImage: {}\nAt: {}\nOther: {}",
            self.text, self.image, self.at, self.other
        )
    }
}

pub struct CountMsgTypes {}

#[async_trait]
impl Handler<MessageEvent> for CountMsgTypes {
    on_message!(MessageEvent);

    async fn handle(&self, event: MessageEvent, matcher: Matcher<MessageEvent>) {
        let mut counts = Counts::default();

        for segment in event.get_message() {
            match segment {
                Message::Reply(reply) => {
                    let Ok(id) = reply.id.parse() else {
                        continue;
                    };
                    if let Some(message) = matcher.get_msg(id).await {
                        counts.add_all(replied_segments(&message));
                    }
                }
                segment => counts.add(segment),
            }
        }

        matcher
            .send(UniMessage::new().text(&counts.to_string()).build())
            .await;
    }
}

fn replied_segments(message: &RespMessage) -> &[Message] {
    match message {
        RespMessage::Group(g) => &g.message,
        RespMessage::Private(p) => &p.message,
    }
}

pub fn count_msg_types() -> Matcher<MessageEvent> {
    Matcher::new("count_msg_types", CountMsgTypes {})
        .add_rule(rules::is_superuser())
        .add_rule(rules::is_group_message_event())
}
