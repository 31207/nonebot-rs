use nonebot_rs;
use mybot::{myshit,notice_test};
fn main() {
    let mut nb = nonebot_rs::Nonebot::new();
    let mut matchers = nonebot_rs::Matchers::new_empty();
    matchers
        //    .add_message_matcher(nonebot_rs::builtin::echo::echo2())
        //    .add_message_matcher(nonebot_rs::builtin::echo::echo())
        .add_message_matcher(nonebot_rs::builtin::rcnb::rcnb())
        .add_message_matcher(nonebot_rs::builtin::bot_status::bot_status(None))
        .add_notice_matcher(notice_test::notice_test())
        .add_message_matcher(myshit::myshit());
    nb.add_plugin(matchers);
    nb.run()
}
