// 发送图片，语音等文件时用 FileType 枚举
// FileType 有两种变体，Path、Url分别对应本地路径和网络链接
// use nonebot_rs::message::FileType;
use builtin_plugins::matcher::prelude::*;
use nonebot_rs::message::UniMessage;

// tokio 异步互斥锁，用于在异步环境下保护共享数据
use tokio::sync::Mutex;

// tracing 日志库，用于记录日志信息
use tracing::{Level, event};

// 定义一个消息事件处理器结构体 MsgEventTest
// 该结构体包含一个异步互斥锁 counter，用于计数用户发送的消息数量
// 每当有消息事件触发时，handle 方法会被调用，记录日志并回复用户发送的消息数量
pub struct MsgEventTest {
    counter: Mutex<usize>,
}

// Rust trait 方法默认 不支持 async fn
// 需要使用 async_trait 宏来实现异步方法
#[async_trait]
impl Handler<MessageEvent> for MsgEventTest {
    // 实现针对MessageEvent的 Handler trait
    /*
       Handler<E> 可针对 Event、MessageEvent、NoticeEvent、RequestEvent、MetaEvent 任意一层实现
       若要处理更细粒度的事件，有两种方式：
         - 用 on_notice!(Essence) / on_event!(...) 宏或 rules::is_notice_type("essence") 过滤
         - 在 handle 方法内对 event 进行匹配，例如：
           match event { MessageEvent::Private(p) => {...}, MessageEvent::Group(g) => {...} }

       具体示例见 bot_example/src/fine_event_test.rs
       可以查看 nonebot_rs/src/event.rs 了解所有事件定义
    */

    /*
       首先需要实现 match_ 方法用于匹配消息事件，在方法中对消息文本进行各种判断
       比如是否包含某个关键词，是否以某个命令开头等
       match_ 返回 Some(Target) 表示匹配成功，handle 将收到该 Target；返回 None 表示不匹配
       不细分类型时 Target 通常就是 E 本身（也可以直接写 type Target = MessageEvent;）

       手动实现的例子：
       type Target = MessageEvent;
       fn match_(&self, event: &mut MessageEvent) -> Option<MessageEvent> {
           Some(event.clone())
       }

       一般不需要手写，on_message! 宏用于简化通配匹配（Target = E）
       除此之外，还有 on_command!、on_start_with! 等宏
       需要把 Target 细化为子事件时，用 on_notice!(Essence)、on_private_message!()、on_group_message!() 等
    */
    on_message!(MessageEvent);

    // 然后，实现 handle 方法用于处理匹配到的消息事件
    // 可以观察一下各类消息如何发送，如何输出日志等
    async fn handle(&self, event: MessageEvent, matcher: Matcher<MessageEvent>) {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        matcher
            .send(
                UniMessage::new()
                    .at(event.get_user_id())
                    .text(format!("你已经发了{}条消息了", counter).as_str())
                    // .image(FileType::Path(String::from(
                    //     "/home/dustwind/Pictures/1.gif",
                    // )))
                    .text("good")
                    .build(),
            )
            .await;
        event!(
            Level::INFO,
            "用户 {} 发送了消息，当前计数为 {}",
            event.get_user_id(),
            *counter
        );
    }
}

// 最后，定义一个函数用于创建该消息事件处理器的 Matcher 实例
// 并添加一个超级用户规则和一个私聊规则，只有超级用户发送的私聊消息才会被该 Handler 处理
// 可以查看nonebot_rs/src/buildin/rules.rs来查看一些内置的规则
// 当然也可以实现自己的规则，亦或是全权交由match_匹配
// 这里推荐：match_方法匹配特定消息内容，rule匹配特定消息来源（发送者，群聊）以及消息类型，handle处理回复逻辑，写日志等
// 但是如果需要在一个Matcher内处理多个消息类型，还是推荐用match在handle方法里处理吧（参见bot_example/src/notice_test.rs)
pub fn msg_event_test() -> Matcher<MessageEvent> {
    Matcher::new(
        "MsgEventTest", // Matcher 名称, 必须唯一
        MsgEventTest {
            // 初始化结构体里的变量
            counter: Mutex::new(0),
        },
    )
    .add_rule(rules::is_superuser())
    .add_pre_matcher(prematchers::to_me())
}
