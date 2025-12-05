#[doc(hidden)]
pub mod macros;
/// 内建 PreMatcher 函数
pub mod prematchers;
/// 内建 rules
pub mod rules;

use tracing::{event, Level};

#[doc(hidden)]
pub fn resp_logger(resp: &crate::api_resp::ApiResp) {
    if &resp.status == "ok" {
        event!(Level::DEBUG, "{} success", resp.echo);
    } else {
        event!(Level::INFO, "{} failed", resp.echo);
    }
}
