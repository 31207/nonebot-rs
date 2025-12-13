use nonebot_rs;
use mybot::*;

fn main() {
    let mut nb = nonebot_rs::Nonebot::new();
    let mut matchers = nonebot_rs::Matchers::new_empty();
    matchers.add_message_matchers(vec![
        meme_parser::meme_parser(),
        helper::helper(),
        ]);
    nb.add_plugin(matchers);
    nb.run()
}
