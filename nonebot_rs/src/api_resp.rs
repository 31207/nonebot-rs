use crate::utils::id_deserializer;
use serde::{Deserialize, Serialize};

/// Onebot Api 响应根结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResp {
    pub status: String,
    pub retcode: i32,
    pub data: RespData,
    pub wording: String,
    pub message: String,
    pub echo: String,
}

// impl ApiResp {
//     pub fn get_date<D>(&self) -> Option<D> {
//         match self.data {
//             RespData::MessageId(d) => Some(d),
//             _ => None,
//         }
//     }
// }

/// Onebot Api 响应 data 字段
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RespData {
    None,
    MessageId(RespMessageId),
    Message(RespMessage),
    Messages(RespMessages),
    LoginInfo(RespLoginInfo),
    StrangerInfo(RespStrangerInfo),
    FriendList(Vec<RespFriendListItem>),
    GroupInfo(RespGroupInfo),
    GroupList(Vec<RespGroupListItem>),
    GroupMemberInfo(RespGroupMemberInfo),
    GroupMemberList(Vec<RespGroupMember>),
    GroupHonorInfo(RespGroupHonorInfo),
    Cookies(RespCookies),
    ScrfToken(RespScrfToken),
    Credentials(RespCredentials),
    File(RespFile),
    SendCheck(RespSendCheck),
    Status(crate::event::Status),
    VersionInfo(RespVersionInfo),
}

/// message_id 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)] // 严格要求只包含 message_id 字段
pub struct RespMessageId {
    pub message_id: i32,
}

/// get_msg 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "message_type")]
pub enum RespMessage {
    /// 响应数据为私聊消息
    #[serde(rename = "private")]
    Private(RespPrivateMessage),

    /// 响应数据为群消息
    #[serde(rename = "group")]
    Group(RespGroupMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespPrivateMessage {
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    pub time: i32,
    pub message_id: i32,
    pub message_seq: i32,
    pub real_id: i32,
    pub sender: crate::event::PrivateSender,
    pub raw_message: String,
    pub font: i32,
    pub sub_type: String,
    pub message: Vec<crate::message::Message>,
    pub message_format: String,
    pub post_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespGroupMessage {
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    pub time: i32,
    pub message_id: i32,
    pub message_seq: i32,
    pub raw_message: String,
    pub font: i32,
    pub sub_type: Option<String>,
    pub message: Vec<crate::message::Message>,
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    pub sender: crate::event::GroupSender,
    pub group_name: String,
}
/// get_forward_msg 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RespMessages {
    pub message: Vec<crate::message::Message>,
}

/// get_login_info 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespLoginInfo {
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    pub nickname: String,
}

/// get_stranger_info 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespStrangerInfo {
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    pub nickname: String,
    pub sex: String,
    pub age: i32,
}

/// get_group_info 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespGroupInfo {
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    pub group_name: String,
    pub member_count: i32,
    pub max_member_count: i32,
    pub group_create_time: i32,
    pub avatar_url: String,
}

/// get_group_member_info 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespGroupMemberInfo {
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    pub nickname: String,
    pub card: String,
    pub sex: String,
    pub age: i32,
    pub area: String,
    pub join_time: i32,
    pub last_sent_time: i32,
    pub level: String,
    pub role: String,
    pub unfriendly: bool,
    pub title: String,
    pub title_expire_time: i32,
    pub card_changeable: bool,
}

/// get_group_honor_info 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespGroupHonorInfo {
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    pub current_talkative: Option<RespCurrentTalkative>,
    pub talkative_list: Option<Vec<RespHonorItem>>,
    pub performer_list: Option<Vec<RespHonorItem>>,
    pub legend_list: Option<Vec<RespHonorItem>>,
    pub strong_newbie_list: Option<Vec<RespHonorItem>>,
    pub emotion_list: Option<Vec<RespHonorItem>>,
}

/// get_cookies 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespCookies {
    pub cookies: String,
}

/// get_csrf_token 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespScrfToken {
    pub token: i32,
}

/// get_credentials 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespCredentials {
    pub cookies: String,
    pub token: i32,
}

/// get_recode && get_image 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespFile {
    pub file: String,
}

/// can_send_image && can_send_record 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespSendCheck {
    pub yes: bool,
}

/// get_version_info 响应数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespVersionInfo {
    pub app_name: String,
    pub app_version: String,
    pub protocol_version: String,
}

/// get_friend_list 响应数组成员
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespFriendListItem {
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    pub nickname: String,
    pub remark: String,
}

/// get_group_list 响应数组成员
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespGroupListItem {
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    pub group_name: String,
    pub member_count: i32,
    pub max_member_count: i32,
}

/// get_group_member_list 响应数组成员
#[derive(Debug, Serialize, Deserialize, Clone)] // need check
pub struct RespGroupMember {
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    pub nickname: String,
    pub card: String,
    pub sex: String,
    pub age: i32,
    pub join_time: i32,
    pub last_sent_time: i32,
    pub level: String,
    pub role: String,
    pub unfriendly: bool,
    pub card_changeable: bool,
}

/// get_group_honor_info 相关
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespCurrentTalkative {
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    pub nickname: String,
    pub avatar: String,
    pub day_count: i32,
}

/// get_group_honor_info 相关
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RespHonorItem {
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    pub nickname: String,
    pub avatar: String,
    pub description: String,
}
