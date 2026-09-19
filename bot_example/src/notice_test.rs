use nonebot_rs::message::UniMessage;
use builtin_plugins::matcher::prelude::*;
// 测试 notice 事件是否能响应
//
// 这里演示在一个 Handler 内用 match 处理多种通知类型；
// 若只关心单一子类型，可用 on_notice!(Essence) 把 Target 细化，handle 免解包（参见 fine_event_test.rs）

#[doc(hidden)]
#[derive(Clone)]
pub struct NoticeTest {}

#[doc(hidden)]
#[async_trait]
impl Handler<NoticeEvent> for NoticeTest {
    on_message!(NoticeEvent);
    async fn handle(&self, event: NoticeEvent, matcher: Matcher<NoticeEvent>) {
        match event {
            NoticeEvent::Notify(m) => {
                matcher
                    .send(
                        UniMessage::new()
                            .text(
                                format!(
                                    "收到通知事件: {}",
                                    m.user_id.as_deref().unwrap_or("")
                                )
                                .as_str(),
                            )
                            .build(),
                    )
                    .await;
            }
            NoticeEvent::GroupRecall(m) => {
                matcher
                    .send(
                        UniMessage::new()
                            .text(format!("收到群消息撤回事件: {}", m.user_id).as_str())
                            .build(),
                    )
                    .await;
            }
            NoticeEvent::FriendRecall(m) => {
                matcher
                    .send(
                        UniMessage::new()
                            .text(format!("收到好友消息撤回事件: {}", m.user_id).as_str())
                            .build(),
                    )
                    .await;
            }
            NoticeEvent::GroupIncrease(m) => {
                matcher
                    .send(
                        UniMessage::new()
                            .text(format!("收到群成员增加事件: {}", m.user_id).as_str())
                            .build(),
                    )
                    .await;
            }
            NoticeEvent::GroupDecrease(m) => {
                matcher
                    .send(
                        UniMessage::new()
                            .text(format!("收到群成员减少事件: {}", m.user_id).as_str())
                            .build(),
                    )
                    .await;
            }
            NoticeEvent::GroupBan(m) => {
                matcher
                    .send(
                        UniMessage::new()
                            .text(format!("收到群成员禁言事件: {}", m.user_id).as_str())
                            .build(),
                    )
                    .await;
            }

            _ => {}
        }
    }
}

pub fn notice_test() -> Matcher<NoticeEvent> {
    Matcher::new("NoticeTest", NoticeTest {}).add_rule(rules::is_superuser())
}

