use nonebot_rs::{matcher::prelude::*, message::UniMessage};
use tracing::{event, Level};
use nonebot_rs::message::FileType;
// 自己写个命令

#[doc(hidden)]
#[derive(Clone)]
pub struct Myshit {}

#[doc(hidden)]
#[async_trait]
impl Handler<MessageEvent> for Myshit {
    on_message!(MessageEvent);
    async fn handle(&self, event: MessageEvent, matcher: Matcher<MessageEvent>) {
        event!(Level::INFO, "有人在发消息:{:?}", event.get_message());
        matcher.send(UniMessage::new()
        .at(event.get_user_id()).text("我草泥马")
        .image(FileType::Path(String::from("/home/dustwind/Pictures/1.gif")))
        .text("nimamama!")
        .build())
        .await;
    }

}

pub fn myshit() -> Matcher<MessageEvent> {
    Matcher::new("Myshit", Myshit {}).add_rule(rules::is_superuser())
}