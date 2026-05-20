use super::Matcher;
use nonebot_rs::onebot_apis;
use nonebot_rs::event::SelfId;
use colored::*;
use tracing::{event, Level};

macro_rules! api_no_resp {
    ($fn_name:ident, $struct_name:tt, $($param:ident: $param_type:ty),+) => {
        pub async fn $fn_name(&self, $($param: $param_type,)*) {
            if let Some(bot) = &self.bot {
                bot.$fn_name($($param,)*).await
            } else {
                event!(
                    Level::ERROR,
                    "Calling api {} {}",
                    stringify!($fn_name).blue(),
                    "with unbuilt matcher!".red()
                );
            }
        }
    };
}

macro_rules! api_resp {
    ($fn_name:ident, $struct_name:tt, $resp_variant:tt, $resp_type:ty,) => {
        pub async fn $fn_name(&self) -> Option<$resp_type> {
            if let Some(bot) = &self.bot {
                bot.$fn_name().await
            } else {
                event!(
                    Level::ERROR,
                    "Calling api {} {}",
                    stringify!($fn_name).blue(),
                    "with unbuilt matcher!".red()
                );
                None
            }
        }
    };
    ($fn_name:ident, $struct_name:tt, $resp_variant:tt, $resp_type:ty, $($param:ident: $param_type:ty),+) => {
        pub async fn $fn_name(&self, $($param: $param_type,)*) -> Option<$resp_type> {
            if let Some(bot) = &self.bot {
                bot.$fn_name($($param,)*).await
            } else {
                event!(
                    Level::ERROR,
                    "Calling api {} {}",
                    stringify!($fn_name).blue(),
                    "with unbuilt matcher!".red()
                );
                None
            }
        }
    };
}

impl<E> Matcher<E>
where
    E: Clone + SelfId + Send,
{
    /// 请求 Onebot Api，不等待 Onebot 返回
    pub async fn call_api(&self, api: nonebot_rs::api::Api) {
        if let Some(bot) = &self.bot {
            bot.call_api(api).await;
        } else {
            event!(
                Level::ERROR,
                "{}",
                "Calling api with unbuilt matcher!".red()
            );
        }
    }

    /// 请求 Onebot Api，等待 Onebot 返回项（30s 后 timeout 返回 None）
    pub async fn call_api_resp(&self, api: nonebot_rs::api::Api) -> Option<nonebot_rs::api_resp::ApiResp> {
        if let Some(bot) = &self.bot {
            bot.call_api_resp(api).await
        } else {
            event!(
                Level::ERROR,
                "{}",
                "Calling api with unbuilt matcher!".red()
            );
            None
        }
    }

    onebot_apis!();
}
