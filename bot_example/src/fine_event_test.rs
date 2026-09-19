// 细粒度事件匹配示例
use builtin_plugins::matcher::prelude::*;
use tracing::{event, Level};

/// 示例一：`Handler<NoticeEvent>` + `on_notice!` 宏
///
/// `on_notice!(Essence)` 把 `Handler::Target` 细化为 `EssenceNoticeEvent`，
/// handle 直接收到具体类型，无需再解包
pub struct EssenceTest {}

#[async_trait]
impl Handler<NoticeEvent> for EssenceTest {
    on_notice!(Essence);

    async fn handle(&self, event: EssenceNoticeEvent, _: Matcher<NoticeEvent>) {
        event!(
            Level::INFO,
            "[EssenceTest] 群 {} 的消息 {} 被 {} {}了精华",
            event.group_id,
            event.message_id,
            event.operator_id,
            if event.sub_type == "add" { "设" } else { "取消" },
        );
    }
}

pub fn essence_test() -> Matcher<NoticeEvent> {
    Matcher::new("EssenceTest", EssenceTest {})
}

/// 示例二：`Handler<Event>` 匹配任意粒度的事件
///
/// `set_block(false)` 表示匹配后事件仍会继续进入分类 matcher；
/// 默认为 true 时，event 层匹配成功将不再进入 message/notice 等分类 matcher
pub struct FineEventTest {}

#[async_trait]
impl Handler<Event> for FineEventTest {
    on_event!(
        Event::Notice(NoticeEvent::FriendAdd(_))
            | Event::Notice(NoticeEvent::GroupAdmin(_))
            | Event::MessageSent(_)
    );

    // message_sent 默认不进入任何 matcher，需要显式开启
    fn match_message_sent(&self) -> bool {
        true
    }

    async fn handle(&self, event: Event, _: Matcher<Event>) {
        match event {
            Event::Notice(NoticeEvent::FriendAdd(f)) => {
                event!(Level::INFO, "[FineEventTest] 新好友: {}", f.user_id)
            }
            Event::Notice(NoticeEvent::GroupAdmin(g)) => {
                event!(
                    Level::INFO,
                    "[FineEventTest] 群 {} 中 {} {}管理员",
                    g.group_id,
                    g.user_id,
                    if g.sub_type == "set" { "被设置" } else { "被取消" },
                )
            }
            Event::MessageSent(m) => {
                event!(
                    Level::INFO,
                    "[FineEventTest] Bot 发送消息: {}",
                    m.get_raw_message()
                )
            }
            _ => {}
        }
    }
}

pub fn fine_event_test() -> Matcher<Event> {
    Matcher::new("FineEventTest", FineEventTest {}).set_block(false)
}
