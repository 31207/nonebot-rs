use bot_example::*;
use nonebot_rs;
fn main() {
    let mut nb = nonebot_rs::Nonebot::new();
    let mut matchers = nonebot_rs::Matchers::new_empty();
    matchers
        //    .add_message_matcher(nonebot_rs::builtin::echo::echo2())
        //    .add_message_matcher(nonebot_rs::builtin::echo::echo())
        .add_message_matcher(rcnb::rcnb())
        .add_message_matcher(bot_status::bot_status(None))
        .add_notice_matcher(notice_test::notice_test())
        .add_message_matcher(msg_event_test::msg_event_test());
    nb.add_plugin(matchers);
    nb.run()
}
