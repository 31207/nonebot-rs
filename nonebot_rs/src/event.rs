use crate::message::Message;
use crate::utils::{id_deserializer, option_id_deserializer};
use serde::{Deserialize, Serialize};

/// WebSocket 接受数据枚举 Event || ApiResp
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecvItem {
    Event(Event),
    ApiResp(crate::api_resp::ApiResp),
}

/// Onebot 事件
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "post_type")]
pub enum Event {
    /// 消息事件
    #[serde(rename = "message")]
    Message(MessageEvent),

    /// 机器人自己发送的消息事件
    #[serde(rename = "message_sent")]
    MessageSent(MessageEvent),

    /// 通知事件
    #[serde(rename = "notice")]
    Notice(NoticeEvent),

    /// 请求事件
    #[serde(rename = "request")]
    Request(RequestEvent),

    /// 元事件
    #[serde(rename = "meta_event")]
    Meta(MetaEvent),

    /// Nonebot 内部事件
    #[serde(skip)]
    Nonebot(NbEvent),
}

/// Nonebot Event
#[derive(Debug, Clone)]
pub enum NbEvent {
    BotConnect { bot: crate::Bot },
    BotDisconnect { bot: crate::Bot },
}

/// 消息事件
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "message_type")]
pub enum MessageEvent {
    /// 私聊事件
    #[serde(rename = "private")]
    Private(PrivateMessageEvent),

    /// 群消息事件
    #[serde(rename = "group")]
    Group(GroupMessageEvent),
}

impl MessageEvent {
    /// 消息事件时间戳
    #[allow(dead_code)]
    pub fn get_time(&self) -> i64 {
        match self {
            MessageEvent::Private(p) => p.time,
            MessageEvent::Group(g) => g.time,
        }
    }

    /// 消息事件字符串格式消息
    #[allow(dead_code)]
    pub fn get_raw_message(&self) -> &str {
        match self {
            MessageEvent::Private(p) => &p.raw_message,
            MessageEvent::Group(g) => &g.raw_message,
        }
    }

    /// 消息事件设置字符串格式消息
    #[allow(dead_code)]
    pub fn set_raw_message(&mut self, new_raw_message: String) {
        match self {
            MessageEvent::Private(p) => {
                p.raw_message = new_raw_message;
            }
            MessageEvent::Group(g) => {
                g.raw_message = new_raw_message;
            }
        }
    }

    /// 消息事件数组格式消息
    #[allow(dead_code)]
    pub fn get_message(&self) -> &Vec<Message> {
        match self {
            MessageEvent::Private(p) => &p.message,
            MessageEvent::Group(g) => &g.message,
        }
    }

    /// 消息事件发送者昵称
    #[allow(dead_code)]
    pub fn get_sender_nickname(&self) -> &str {
        match self {
            MessageEvent::Private(p) => &p.sender.nickname,
            MessageEvent::Group(g) => &g.sender.nickname,
        }
    }
}

/// 私聊消息事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateMessageEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 消息子类型
    pub sub_type: String,
    /// 消息 ID
    pub message_id: i32,
    pub message_seq: i32,
    /// 发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// Array 消息内容
    pub message: Vec<Message>,
    pub message_format: String,
    pub raw_pb: String,
    /// 原生消息内容
    pub raw_message: String,
    /// 字体
    pub font: i32,
    /// 发送者消息
    pub sender: PrivateSender,
}

/// 私聊消息事件发送者
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateSender {
    /// 发送者 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 昵称
    pub nickname: String,
}

/// 群消息事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupMessageEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 消息子类型
    pub sub_type: String,
    /// 消息 ID
    pub message_id: i32,
    /// ???
    pub message_seq: i32,
    pub message_format: String,
    pub raw_pb: String,

    /// 群消息群号
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    pub group_name: String,
    /// 发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 匿名消息 非匿名消息为空
    pub anonymous: Option<Anonymous>,
    /// Array 消息内容
    pub message: Vec<Message>,
    /// 原生消息内容
    pub raw_message: String,
    /// 字体
    pub font: i32,
    /// 发送者消息
    pub sender: GroupSender,
}

/// 群消息事件发送者
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupSender {
    /// 发送者 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 昵称
    pub nickname: String,
    /// 群名片|备注
    pub card: String,
    /// 性别 male|female|unkown
    // pub sex: String,
    /// 年龄
    // pub age: i32,
    /// 地区
    // pub area: String,
    /// 成员等级
    // pub level: String,
    /// 角色 owner|admin|member
    pub role: String,
    /// 专属头衔
    pub title: String,
}

/// 消息事件匿名字段
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Anonymous {
    /// 匿名用户 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub id: String,
    /// 匿名用户名称
    pub name: String,
    /// 匿名用户 flag
    pub flag: String,
}

/// 通知事件
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "notice_type")]
pub enum NoticeEvent {
    #[serde(rename = "notify")]
    Notify(NotifyNoticeEvent),

    #[serde(rename = "friend_recall")]
    FriendRecall(FriendRecallNoticeEvent),

    #[serde(rename = "group_recall")]
    GroupRecall(GroupRecallNoticeEvent),

    #[serde(rename = "group_increase")]
    GroupIncrease(GroupIncreaseNoticeEvent),

    #[serde(rename = "group_decrease")]
    GroupDecrease(GroupDecreaseNoticeEvent),

    #[serde(rename = "group_ban")]
    GroupBan(GroupBanNoticeEvent),

    #[serde(rename = "group_msg_emoji_like")]
    GroupMessageEmojiLike(GroupMessageEmojiLikeNoticeEvent),

    #[serde(rename = "group_card")]
    GroupCard(GroupCardNoticeEvent),

    #[serde(rename = "group_upload")]
    GroupUpload(GroupUploadNoticeEvent),

    #[serde(rename = "essence")]
    Essence(EssenceNoticeEvent),

    #[serde(rename = "friend_add")]
    FriendAdd(FriendAddNoticeEvent),

    #[serde(rename = "group_admin")]
    GroupAdmin(GroupAdminNoticeEvent),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotifyNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 事件子类型
    pub sub_type: Option<String>,
    /// 发送者 ID
    #[serde(default, deserialize_with = "option_id_deserializer")]
    pub user_id: Option<String>,
    /// 被戳者 ID
    #[serde(default, deserialize_with = "option_id_deserializer")]
    pub target_id: Option<String>,
    /// 群号
    #[serde(default, deserialize_with = "option_id_deserializer")]
    pub group_id: Option<String>,
    /// 操作者 ID (sub_type = "profile_like" 时)
    #[serde(default, deserialize_with = "option_id_deserializer")]
    pub operator_id: Option<String>,
    /// 操作者昵称 (sub_type = "profile_like" 时)
    #[serde(default)]
    pub operator_nick: Option<String>,
    /// 点赞次数 (sub_type = "profile_like" 时)
    #[serde(default)]
    pub times: Option<i64>,
    /// 原始json数据
    #[serde(default)]
    pub raw_info: Option<serde_json::Value>,
    /// 头衔 (sub_type = "title" 时)
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FriendRecallNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 被撤回的消息 ID
    pub message_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupRecallNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 群号
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    /// 发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 被撤回的消息 ID
    pub message_id: i64,
    /// 操作者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub operator_id: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupIncreaseNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 群号
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    /// 发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 操作者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub operator_id: String,
    /// 子类型
    pub sub_type: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupDecreaseNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 群号
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    /// 发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 操作者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub operator_id: String,
    /// 子类型
    pub sub_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupBanNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 群号
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    /// 发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 操作者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub operator_id: String,
    /// 禁言时长，单位秒
    pub duration: i64,
    /// 子类型
    pub sub_type: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupMessageEmojiLikeNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 群号
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    /// 发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// is add
    pub is_add: bool,
    /// likes
    pub likes: serde_json::Value,
    /// message_id
    pub message_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupCardNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 群号
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    /// 发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 新群名片
    pub card_new: String,
    /// 旧群名片
    pub card_old: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupUploadNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 群号
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    /// 发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 上传文件信息
    pub file: GroupUploadFile,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupUploadFile {
    /// 文件 ID
    pub id: String,
    /// 文件名
    pub name: String,
    /// 文件大小
    pub size: i64,
    /// busid
    pub busid: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EssenceNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 被设为精华的消息 ID
    pub message_id: i64,
    /// 消息发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub sender_id: String,
    /// 子类型 add|delete
    pub sub_type: String,
    /// 群号
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    /// 发送者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 操作者 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub operator_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FriendAddNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 添加好友的用户 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupAdminNoticeEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 子类型 set|unset
    pub sub_type: String,
    /// 群号
    #[serde(deserialize_with = "id_deserializer")]
    pub group_id: String,
    /// 被设置/取消管理员的用户 ID
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 操作者 ID
    #[serde(default, deserialize_with = "option_id_deserializer")]
    pub operator_id: Option<String>,
}
/// 请求事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 请求类型
    pub request_type: String,
    /// 发送请求的 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub user_id: String,
    /// 验证信息
    pub comment: String,
    /// 请求 flag
    pub flag: String,
    /// 请求子类型
    pub sub_type: Option<String>,
    /// 群号
    #[serde(deserialize_with = "option_id_deserializer")]
    #[serde(default)]
    pub group_id: Option<String>,
}

/// 元事件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetaEvent {
    /// Event 时间戳
    pub time: i64,
    /// 收到事件的机器人 QQ 号
    #[serde(deserialize_with = "id_deserializer")]
    pub self_id: String,
    /// 元事件类型 lifecycle|heartbeat
    pub meta_event_type: String,
    /// 事件子类型
    pub sub_type: Option<String>,
    /// 状态信息
    pub status: Option<Status>,
    /// 下次心跳间隔，单位毫秒
    pub interval: Option<i64>,
}

#[test]
fn de_test() {
    let test_str = "{\"group_id\":101,\"message_id\":111,\"notice_type\":\"group_recall\",\"operator_id\":11,\"post_type\":\"notice\",\"self_id\":11,\"time\":1631193409,\"user_id\":11}\n";
    let _meta: Event = serde_json::from_str(test_str).unwrap();
}

#[test]
fn de_group_mface_event() {
    let json = r#"{
        "self_id": 3579148268,
        "user_id": 1635738402,
        "time": 1779283951,
        "message_id": 1335628842,
        "message_seq": 478145,
        "message_type": "group",
        "sender": {
            "user_id": 1635738402,
            "nickname": "破损的海兔无人机",
            "card": "",
            "role": "member",
            "level": "37",
            "title": ""
        },
        "raw_message": "[CQ:mface,summary=&#91;摸头&#93;,url=https://gxh.vip.qq.com/club/item/parcel/item/3a/3a181abe476219e7a04163eab80fa274/raw300.gif,emoji_id=3a181abe476219e7a04163eab80fa274,emoji_package_id=244959,key=ea229eea083f3bbb]",
        "font": 14,
        "sub_type": "normal",
        "message": [
            {
                "type": "mface",
                "data": {
                    "summary": "[摸头]",
                    "url": "https://gxh.vip.qq.com/club/item/parcel/item/3a/3a181abe476219e7a04163eab80fa274/raw300.gif",
                    "emoji_id": "3a181abe476219e7a04163eab80fa274",
                    "emoji_package_id": 244959,
                    "key": "ea229eea083f3bbb"
                }
            }
        ],
        "message_format": "array",
        "post_type": "message",
        "raw_pb": "",
        "group_id": 650342021,
        "group_name": "[2.07.2] Beebeeblock: 蜂蜂空坊交流群"
    }"#;
    let event: Event = serde_json::from_str(json).unwrap();
    match event {
        Event::Message(MessageEvent::Group(g)) => {
            assert_eq!(g.group_id, "650342021");
            assert_eq!(g.user_id, "1635738402");
            assert_eq!(g.sender.nickname, "破损的海兔无人机");
            assert_eq!(g.message.len(), 1);
            match &g.message[0] {
                crate::message::Message::Mface(m) => {
                    assert_eq!(m.summary, "[摸头]");
                    assert_eq!(m.emoji_id, "3a181abe476219e7a04163eab80fa274");
                    assert_eq!(m.emoji_package_id, 244959);
                }
                _ => panic!("expected Mface message segment"),
            }
        }
        _ => panic!("expected group message event"),
    }
}

#[test]
fn de_file_message_event() {
    let json = r#"{
        "self_id": 1692038362,
        "user_id": 3966708213,
        "time": 1787989388,
        "message_id": 909916862,
        "message_seq": 5319671,
        "message_type": "group",
        "sender": {
            "user_id": 3966708213,
            "nickname": "Jeryz",
            "card": "[有人@我]Jeryz",
            "role": "member",
            "level": "24",
            "title": "我好可爱！"
        },
        "raw_message": "[CQ:file,file=QQ20260829-154116.mp4,url=,file_id=/ad295eef-f08f-4ec1-8c1f-bd158e513c17,path=,file_size=16717747]",
        "font": 14,
        "sub_type": "normal",
        "message": [
            {
                "type": "file",
                "data": {
                    "file": "QQ20260829-154116.mp4",
                    "url": "",
                    "file_id": "/ad295eef-f08f-4ec1-8c1f-bd158e513c17",
                    "path": "",
                    "file_size": "16717747"
                }
            }
        ],
        "message_format": "array",
        "post_type": "message",
        "raw_pb": "",
        "group_id": 253461266,
        "group_name": "OSU!Mania-笨笨萌新交流群"
    }"#;
    let event: Event = serde_json::from_str(json).unwrap();
    match event {
        Event::Message(MessageEvent::Group(g)) => {
            assert_eq!(g.group_id, "253461266");
            assert_eq!(g.message.len(), 1);
            match &g.message[0] {
                crate::message::Message::File(f) => {
                    assert_eq!(f.file, "QQ20260829-154116.mp4");
                    assert_eq!(f.file_id.as_deref(), Some("/ad295eef-f08f-4ec1-8c1f-bd158e513c17"));
                    assert_eq!(f.file_size.as_deref(), Some("16717747"));
                }
                _ => panic!("expected File message segment"),
            }
        }
        _ => panic!("expected group message event"),
    }
}

#[test]
fn de_keyboard_markdown_dice_message_event() {
    let json = r#"{
        "self_id": 1692038362,
        "user_id": 2854211260,
        "time": 1787992519,
        "message_id": -280154787,
        "message_seq": 3620502,
        "message_type": "group",
        "sender": {
            "user_id": 2854211260,
            "nickname": "是萌卡喵呢",
            "card": "",
            "role": "member",
            "level": "0",
            "title": ""
        },
        "raw_message": "[CQ:keyboard,rows=...][CQ:markdown,content=...]",
        "font": 14,
        "sub_type": "normal",
        "message": [
            {
                "type": "keyboard",
                "data": {
                    "rows": [
                        {
                            "buttons": [
                                {
                                    "id": "1",
                                    "render_data": {"label": "今日老婆", "visited_label": "今日老婆", "style": 1},
                                    "action": {"type": 2, "permission": {"type": 2, "specify_role_ids": [], "specify_user_ids": []}, "unsupport_tips": "", "data": "今日老婆", "reply": false, "enter": false}
                                }
                            ]
                        }
                    ]
                }
            },
            {"type": "markdown", "data": {"content": "[](%7B%22version%22%3A2%7D)\n![img](https://qqbot.ugcimg.cn/xxx)"}},
            {"type": "dice", "data": {"result": "5"}},
            {"type": "at", "data": {"qq": "1170672908", "name": "108"}}
        ],
        "message_format": "array",
        "post_type": "message",
        "raw_pb": "",
        "group_id": 253461266,
        "group_name": "test"
    }"#;
    let event: Event = serde_json::from_str(json).unwrap();
    match event {
        Event::Message(MessageEvent::Group(g)) => {
            assert_eq!(g.message.len(), 4);
            match &g.message[0] {
                crate::message::Message::Keyboard(k) => {
                    assert!(k.rows.is_array());
                }
                _ => panic!("expected Keyboard message segment"),
            }
            match &g.message[1] {
                crate::message::Message::Markdown(m) => {
                    assert!(m.content.contains("version"));
                }
                _ => panic!("expected Markdown message segment"),
            }
            match &g.message[2] {
                crate::message::Message::Dice(d) => {
                    assert_eq!(d.result, "5");
                }
                _ => panic!("expected Dice message segment"),
            }
            match &g.message[3] {
                crate::message::Message::At(a) => {
                    assert_eq!(a.qq, "1170672908");
                    assert_eq!(a.name.as_deref(), Some("108"));
                }
                _ => panic!("expected At message segment"),
            }
        }
        _ => panic!("expected group message event"),
    }
}

#[test]
fn de_group_card_and_upload_notice_event() {
    let card = r#"{
        "time": 1787998371,
        "self_id": 1692038362,
        "post_type": "notice",
        "notice_type": "group_card",
        "group_id": 253461266,
        "user_id": 3493682691,
        "card_new": "",
        "card_old": "入口即化柔柔弱弱软软糯糯威威风风浩浩荡荡"
    }"#;
    let event: Event = serde_json::from_str(card).unwrap();
    match event {
        Event::Notice(NoticeEvent::GroupCard(g)) => {
            assert_eq!(g.group_id, "253461266");
            assert_eq!(g.user_id, "3493682691");
            assert_eq!(g.card_old, "入口即化柔柔弱弱软软糯糯威威风风浩浩荡荡");
        }
        _ => panic!("expected group_card notice event"),
    }

    let upload = r#"{
        "time": 1788004958,
        "self_id": 1692038362,
        "post_type": "notice",
        "notice_type": "group_upload",
        "file": {
            "id": "/87481339-3157-4b5c-9ad7-e6adff24f3e3",
            "name": "1b86bbc2046898063501ba1fc9738921.mp4",
            "size": 3685706,
            "busid": 104
        },
        "group_id": 249335821,
        "user_id": 3611925387
    }"#;
    let event: Event = serde_json::from_str(upload).unwrap();
    match event {
        Event::Notice(NoticeEvent::GroupUpload(g)) => {
            assert_eq!(g.group_id, "249335821");
            assert_eq!(g.file.name, "1b86bbc2046898063501ba1fc9738921.mp4");
            assert_eq!(g.file.size, 3685706);
        }
        _ => panic!("expected group_upload notice event"),
    }
}

#[test]
fn de_title_notify_event() {
    let json = r#"{"time":1788097271,"self_id":1692038362,"post_type":"notice","notice_type":"notify","sub_type":"title","title":"前会长","group_id":593888649,"user_id":1428378600}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    match event {
        Event::Notice(NoticeEvent::Notify(n)) => {
            assert_eq!(n.sub_type.as_deref(), Some("title"));
            assert_eq!(n.title.as_deref(), Some("前会长"));
            assert_eq!(n.group_id.as_deref(), Some("593888649"));
            assert_eq!(n.user_id.as_deref(), Some("1428378600"));
            assert!(n.target_id.is_none());
        }
        _ => panic!("expected notify title notice event"),
    }
}

#[test]
fn de_profile_like_notify_event() {
    let json = r#"{"time":1789812789,"self_id":1692038362,"post_type":"notice","notice_type":"notify","sub_type":"profile_like","operator_id":516965699,"operator_nick":"施君纹吉","times":5}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    match event {
        Event::Notice(NoticeEvent::Notify(n)) => {
            assert_eq!(n.sub_type.as_deref(), Some("profile_like"));
            assert_eq!(n.operator_id.as_deref(), Some("516965699"));
            assert_eq!(n.operator_nick.as_deref(), Some("施君纹吉"));
            assert_eq!(n.times, Some(5));
            assert!(n.user_id.is_none());
        }
        _ => panic!("expected profile_like notify event"),
    }
}

#[test]
fn de_message_sent_private_file_event() {
    let json = r#"{"self_id":1692038362,"user_id":1692038362,"time":1788282004,"message_id":370745486,"message_seq":0,"message_type":"private","sender":{"user_id":1692038362,"nickname":""},"raw_message":"[CQ:file,file=まめきぷ.png,url=,file_id=,path=,file_size=1630612]","font":14,"sub_type":"friend","message":[{"type":"file","data":{"file":"まめきぷ.png","url":"","file_id":"","path":"","file_size":"1630612"}}],"message_format":"array","post_type":"message_sent","raw_pb":"","target_id":0}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    match event {
        Event::MessageSent(MessageEvent::Private(p)) => {
            assert_eq!(p.self_id, "1692038362");
            assert_eq!(p.user_id, "1692038362");
            assert_eq!(p.sub_type, "friend");
            match &p.message[0] {
                crate::message::Message::File(f) => {
                    assert_eq!(f.file, "まめきぷ.png");
                    assert_eq!(f.file_size.as_deref(), Some("1630612"));
                }
                _ => panic!("expected File message segment"),
            }
        }
        _ => panic!("expected message_sent private event"),
    }
}

#[test]
fn de_shake_message_event() {
    let json = r#"{"self_id":1692038362,"user_id":1165663204,"time":1788628939,"message_id":1455634142,"message_seq":7903384,"message_type":"group","sender":{"user_id":1165663204,"nickname":"","card":"George33（ln苦手","role":"member","level":"62","title":""},"raw_message":"[CQ:shake]","font":14,"sub_type":"normal","message":[{"type":"shake","data":{}}],"message_format":"array","post_type":"message","raw_pb":"","group_id":249335821,"group_name":"osu!mania 4K萌新交流群"}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    match event {
        Event::Message(MessageEvent::Group(g)) => {
            assert_eq!(g.group_id, "249335821");
            assert_eq!(g.sender.card, "George33（ln苦手");
            match &g.message[0] {
                crate::message::Message::Shake(_) => {}
                _ => panic!("expected Shake message segment"),
            }
        }
        _ => panic!("expected group message event"),
    }
}

#[test]
fn de_essence_notice_event() {
    let json = r#"{"time":1789617717,"self_id":1692038362,"post_type":"notice","notice_type":"essence","message_id":-279345864,"sender_id":1707235874,"sub_type":"add","group_id":621184446,"user_id":1707235874,"operator_id":2777876404}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    match event {
        Event::Notice(NoticeEvent::Essence(e)) => {
            assert_eq!(e.message_id, -279345864);
            assert_eq!(e.sender_id, "1707235874");
            assert_eq!(e.operator_id, "2777876404");
            assert_eq!(e.group_id, "621184446");
            assert_eq!(e.sub_type, "add");
        }
        _ => panic!("expected essence notice event"),
    }
}

#[test]
fn de_friend_add_notice_event() {
    let json = r#"{"time":1788602602,"self_id":1692038362,"post_type":"notice","notice_type":"friend_add","user_id":1951701741}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    match event {
        Event::Notice(NoticeEvent::FriendAdd(f)) => {
            assert_eq!(f.user_id, "1951701741");
        }
        _ => panic!("expected friend_add notice event"),
    }
}

#[test]
fn de_group_admin_notice_event() {
    let json = r#"{"time":1789466674,"self_id":1692038362,"post_type":"notice","notice_type":"group_admin","sub_type":"set","group_id":671990216,"user_id":3535725493}"#;
    let event: Event = serde_json::from_str(json).unwrap();
    match event {
        Event::Notice(NoticeEvent::GroupAdmin(g)) => {
            assert_eq!(g.sub_type, "set");
            assert_eq!(g.group_id, "671990216");
            assert_eq!(g.user_id, "3535725493");
            assert!(g.operator_id.is_none());
        }
        _ => panic!("expected group_admin notice event"),
    }
}

/// 元事件状态字段
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
    /// 是否在线，None 表示无法查询
    pub online: Option<bool>,
    /// 运行状态是否符合预期
    pub good: bool,
}

/// `get_user_id()` trait
pub trait UserId {
    fn get_user_id(&self) -> String;
}

impl UserId for MessageEvent {
    fn get_user_id(&self) -> String {
        match self {
            MessageEvent::Private(p) => p.user_id.to_string(),
            MessageEvent::Group(g) => g.user_id.to_string(),
        }
    }
}

impl UserId for NoticeEvent {
    fn get_user_id(&self) -> String {
        match self {
            NoticeEvent::Notify(n) => n
                .user_id
                .clone()
                .or_else(|| n.operator_id.clone())
                .unwrap_or_default(),
            NoticeEvent::FriendRecall(f) => f.user_id.clone(),
            NoticeEvent::GroupRecall(g) => g.user_id.clone(),
            NoticeEvent::GroupIncrease(g) => g.user_id.clone(),
            NoticeEvent::GroupDecrease(g) => g.user_id.clone(),
            NoticeEvent::GroupBan(g) => g.user_id.clone(),
            NoticeEvent::GroupMessageEmojiLike(g) => g.user_id.clone(),
            NoticeEvent::GroupCard(g) => g.user_id.clone(),
            NoticeEvent::GroupUpload(g) => g.user_id.clone(),
            NoticeEvent::Essence(e) => e.user_id.clone(),
            NoticeEvent::FriendAdd(f) => f.user_id.clone(),
            NoticeEvent::GroupAdmin(g) => g.user_id.clone(),
        }
    }
}

impl UserId for RequestEvent {
    fn get_user_id(&self) -> String {
        self.user_id.clone()
    }
}

/// `get_self_id()` trait
pub trait SelfId {
    fn get_self_id(&self) -> String;
}

impl SelfId for MessageEvent {
    fn get_self_id(&self) -> String {
        match self {
            MessageEvent::Private(p) => p.self_id.clone(),
            MessageEvent::Group(g) => g.self_id.clone(),
        }
    }
}

impl SelfId for RequestEvent {
    fn get_self_id(&self) -> String {
        self.self_id.clone()
    }
}

impl SelfId for NoticeEvent {
    fn get_self_id(&self) -> String {
        match self {
            NoticeEvent::Notify(n) => n.self_id.clone(),
            NoticeEvent::FriendRecall(f) => f.self_id.clone(),
            NoticeEvent::GroupRecall(g) => g.self_id.clone(),
            NoticeEvent::GroupIncrease(g) => g.self_id.clone(),
            NoticeEvent::GroupDecrease(g) => g.self_id.clone(),
            NoticeEvent::GroupBan(g) => g.self_id.clone(),
            NoticeEvent::GroupMessageEmojiLike(g) => g.self_id.clone(),
            NoticeEvent::GroupCard(g) => g.self_id.clone(),
            NoticeEvent::GroupUpload(g) => g.self_id.clone(),
            NoticeEvent::Essence(e) => e.self_id.clone(),
            NoticeEvent::FriendAdd(f) => f.self_id.clone(),
            NoticeEvent::GroupAdmin(g) => g.self_id.clone(),
        }
    }
}

impl SelfId for MetaEvent {
    fn get_self_id(&self) -> String {
        self.self_id.clone()
    }
}

impl SelfId for Event {
    fn get_self_id(&self) -> String {
        match self {
            Event::Message(e) => e.get_self_id(),
            Event::MessageSent(e) => e.get_self_id(),
            Event::Request(e) => e.get_self_id(),
            Event::Notice(e) => e.get_self_id(),
            Event::Meta(e) => e.get_self_id(),
            Event::Nonebot(e) => match e {
                NbEvent::BotConnect { bot } => bot.bot_id.clone(),
                NbEvent::BotDisconnect { bot } => bot.bot_id.clone(),
            },
        }
    }
}

impl GroupBanNoticeEvent {
    pub fn is_ban_or_lift_ban(&self) -> bool {
        self.sub_type == "ban"
    }
}

impl NoticeEvent {
    /// 通知事件的 notice_type 字符串
    pub fn get_notice_type(&self) -> &'static str {
        match self {
            NoticeEvent::Notify(_) => "notify",
            NoticeEvent::FriendRecall(_) => "friend_recall",
            NoticeEvent::GroupRecall(_) => "group_recall",
            NoticeEvent::GroupIncrease(_) => "group_increase",
            NoticeEvent::GroupDecrease(_) => "group_decrease",
            NoticeEvent::GroupBan(_) => "group_ban",
            NoticeEvent::GroupMessageEmojiLike(_) => "group_msg_emoji_like",
            NoticeEvent::GroupCard(_) => "group_card",
            NoticeEvent::GroupUpload(_) => "group_upload",
            NoticeEvent::Essence(_) => "essence",
            NoticeEvent::FriendAdd(_) => "friend_add",
            NoticeEvent::GroupAdmin(_) => "group_admin",
        }
    }
}

impl UserId for Event {
    fn get_user_id(&self) -> String {
        match self {
            Event::Message(e) => e.get_user_id(),
            Event::MessageSent(e) => e.get_user_id(),
            Event::Notice(e) => e.get_user_id(),
            Event::Request(e) => e.get_user_id(),
            Event::Meta(_) => String::new(),
            Event::Nonebot(_) => String::new(),
        }
    }
}
