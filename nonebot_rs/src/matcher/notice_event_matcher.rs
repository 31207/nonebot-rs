use super::Matcher;
use crate::{event::NoticeEvent, message::UniMessage};
use colored::*;
use tracing::{event, Level};

impl Matcher<NoticeEvent> {
    /// 发送纯文本消息
    pub async fn send_text(&self, msg: &str) {
        self.send(UniMessage::new().text(msg).build()).await;
    }
    /// 发送 Vec<Message> 消息
    pub async fn send(&self, msg: Vec<crate::message::Message>) {
        if let (Some(bot), Some(event)) = (&self.bot, &self.event) {
            bot.send_by_notice_event(&event, msg).await;
        } else {
            event!(
                Level::ERROR,
                "{}",
                "Sending msg with unbuilt matcher!".red()
            );
        }
    }
}
