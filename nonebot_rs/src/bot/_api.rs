use crate::onebot_apis;
use crate::{api, RespData};

macro_rules! api_no_resp {
    ($fn_name:ident, $struct_name:tt, $($param:ident: $param_type:ty),+) => {
        pub async fn $fn_name(&self, $($param: $param_type,)*) {
            self.call_api(api::Api::$fn_name(api::$struct_name {
                $($param: $param,)*
            })).await;
        }
    };
}

macro_rules! api_resp {
    ($fn_name:ident, $struct_name:tt, $resp_data:tt, $resp_data_type:ty,) => {
        pub async fn $fn_name(&self) -> Option<$resp_data_type> {
            let resp = self.call_api_resp(api::Api::$fn_name()).await;
            if let RespData::$resp_data(d) = resp?.data {
                Some(d)
            } else {
                None
            }
        }
    };
    ($fn_name:ident, $struct_name:tt, $resp_data:tt, $resp_data_type:ty, $($param:ident: $param_type:ty),+) => {
        pub async fn $fn_name(&self, $($param: $param_type,)*) -> Option<$resp_data_type> {
            let resp = self
                .call_api_resp(api::Api::$fn_name(api::$struct_name {
                    $($param: $param,)*
                }))
                .await;
            if let RespData::$resp_data(d) = resp?.data {
                Some(d)
            } else {
                None
            }
        }
    };
}

impl super::Bot {
    onebot_apis!();
}
