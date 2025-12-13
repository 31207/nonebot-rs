use serde::{Deserialize, Serialize};

/// Onebot 协议消息定义
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    /// 纯文本
    #[serde(rename = "text")]
    Text(Text),

    /// QQ 表情
    #[serde(rename = "face")]
    Face(Face),

    /// 图片
    #[serde(rename = "image")]
    Image(Image),

    /// 语音
    #[serde(rename = "record")]
    Record(Record),

    /// 短视频
    #[serde(rename = "video")]
    Video(Video),

    /// @某人
    #[serde(rename = "at")]
    At(At),

    /// 猜拳魔法表情
    #[serde(rename = "rps")]
    Rps,

    /// 掷骰子魔法表情
    #[serde(rename = "dice")]
    Dice,

    /// 窗口抖动（戳一戳）
    #[serde(rename = "shake")]
    Shake,

    /// 戳一戳
    #[serde(rename = "poke")]
    Poke(Poke),

    /// 匿名发消息
    #[serde(rename = "anonymous")]
    Anonymous,

    /// 链接分享
    #[serde(rename = "share")]
    Share(Share),

    /// 推荐好友|群
    #[serde(rename = "contact")]
    Contact(Contact),

    /// 位置
    #[serde(rename = "location")]
    Lacation(Lacation),

    /// 音乐分享
    #[serde(rename = "music")]
    Music(Music),

    /// 回复
    #[serde(rename = "reply")]
    Reply(Reply),

    /// 合并转发
    #[serde(rename = "forward")]
    Forward(Forward),

    /// 合并转发节点
    #[serde(rename = "node")]
    Node(Node),

    /// XML 消息
    #[serde(rename = "xml")]
    Xml(Xml),

    /// JSON 消息
    #[serde(rename = "json")]
    Json(Json),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Text {
    pub text: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Face {
    /// QQ 表情 ID
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Record {
    /// 语音文件名
    pub file: String,
    /// 是否变声 1|0
    pub magic: Option<u8>,
    /// 语音 URL    
    pub url: Option<String>,
    /// 是否使用缓存文件 1|0
    pub cache: Option<u8>,
    /// 是否使用代理 1|0
    pub proxy: Option<u8>,
    /// 网络文件下载超时 单位秒
    pub timeout: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    /// 图片文件名
    pub file: String,
    /// 图片类型 flash 闪照
    #[serde(rename = "type")]
    pub type_: Option<String>,
    /// 图片 URL
    pub url: Option<String>,
    /// 是否使用缓存文件 1|0
    pub cache: Option<u8>,
    /// 是否使用代理 1|0
    pub proxy: Option<u8>,
    /// 网络文件下载超时 单位秒
    pub timeout: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Video {
    /// 视频文件名
    pub file: String,
    /// 视频 URL
    pub url: Option<String>,
    /// 是否使用缓存文件 1|0
    pub cache: Option<u8>,
    /// 是否使用代理 1|0
    pub proxy: Option<u8>,
    /// 网络文件下载超时 单位秒
    pub timeout: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct At {
    /// @QQ ID all 表示全体
    pub qq: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Poke {
    /// 类型
    #[serde(rename = "type")]
    pub type_: String,
    /// ID
    pub id: String,
    /// 表情名
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Share {
    /// URL
    pub url: String,
    /// 标题
    pub title: String,
    /// 内容描述
    pub content: Option<String>,
    /// 图片 URl
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Contact {
    /// 类型 qq|group
    #[serde(rename = "type")]
    pub type_: String,
    /// QQ号|群号
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lacation {
    /// 纬度
    pub lat: String,
    /// 经度           
    pub lon: String,
    /// 标题  
    pub title: Option<String>,
    /// 内容描述
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Music {
    /// 类型 qq|163|xm|custom
    #[serde(rename = "type")]
    pub type_: String,
    /// 歌曲 ID
    pub id: Option<String>,
    /// 点击后跳转 URL
    pub url: Option<String>,
    /// 歌曲 URL  
    pub audio: Option<String>,
    /// 标题   
    pub title: Option<String>,
    /// 内容描述
    pub content: Option<String>,
    /// 图片 URl
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reply {
    /// 回复的消息 ID
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Forward {
    /// 合并转发 ID
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    /// 转发的消息 ID
    pub id: Option<String>,
    /// 发送者 QQ 号        
    pub user_id: Option<String>,
    /// 发送者昵称   
    pub nickname: Option<String>,
    /// 消息内容     
    pub content: Option<Vec<Message>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Xml {
    /// 回复的消息 ID
    pub data: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Json {
    /// 回复的消息 ID
    pub data: String,
}
// macro_rules! message_builder {
//     ($fn_name: ident, $message_type: tt) => {
//         pub fn $fn_name() -> Message {
//             Message::$message_type
//         }
//     };
//     ($fn_name: ident, $message_type: tt, $param: ident: $param_ty: ty) => {
//         pub fn $fn_name($param: $param_ty) -> Message {
//             Message::$message_type { $param: $param }
//         }
//     };
//     ($fn_name: ident, $message_type: tt, $($param: ident: $param_ty: ty),*) => {
//         pub fn $fn_name($($param: $param_ty,)*) -> Message {
//             Message::$message_type { $($param: $param,)* }
//         }
//     };
// }

impl Message {
    // pub fn text(text: &str) -> Message {
    //     Message::Text {
    //         text: text.to_string(),
    //     }
    // }
    // message_builder!(text, Text, text: String);
    // message_builder!(face, Face, id: String);
    // message_builder!(
    //     image,
    //     Image,
    //     file: String,
    //     type_: Option<String>,
    //     url: Option<String>,
    //     cache: Option<u8>,
    //     proxy: Option<u8>,
    //     timeout: Option<i64>
    // );
    // message_builder!(
    //     record,
    //     Record,
    //     file: String,
    //     magic: Option<u8>,
    //     url: Option<String>,
    //     cache: Option<u8>,
    //     proxy: Option<u8>,
    //     timeout: Option<i64>
    // );
    // message_builder!(
    //     video,
    //     Video,
    //     file: String,
    //     url: Option<String>,
    //     cache: Option<u8>,
    //     proxy: Option<u8>,
    //     timeout: Option<i64>
    // );
    // message_builder!(at, At, qq: String);
    // message_builder!(rps, Rps);
    // message_builder!(dice, Dice);
    // message_builder!(shake, Shake);
    // message_builder!(poke, Poke, type_: String, id: String, name: Option<String>);
    // message_builder!(anonymous, Anonymous);
    // message_builder!(
    //     share,
    //     Share,
    //     url: String,
    //     title: String,
    //     content: Option<String>,
    //     image: Option<String>
    // );
    // message_builder!(contact, Contact, type_: String, id: String);
    // message_builder!(
    //     location,
    //     Lacation,
    //     lat: String,
    //     lon: String,
    //     title: Option<String>,
    //     content: Option<String>
    // );
    // message_builder!(
    //     music,
    //     Music,
    //     type_: String,
    //     id: Option<String>,
    //     url: Option<String>,
    //     audio: Option<String>,
    //     title: Option<String>,
    //     content: Option<String>,
    //     image: Option<String>
    // );
    // message_builder!(reply, Reply, id: String);
    // message_builder!(forward, Forward, id: String);
    // message_builder!(
    //     node,
    //     Node,
    //     id: Option<String>,
    //     user_id: Option<String>,
    //     nickname: Option<String>,
    //     content: Option<Vec<Message>>
    // );
    // message_builder!(xml, Xml, data: String);
    // message_builder!(json, Json, data: String);
}
#[derive(Debug)]
pub enum FileType {
    // url, timeout_seconds
    Url(String, i64),
    // local file path
    Path(String),
    Base64(String),
}

pub struct UniMessage {
    messages: Vec<Message>,
}
impl UniMessage {
    fn load_file_as_base64(path: &str) -> Option<String> {
        use base64::prelude::*;
        use std::fs;
        use tracing::{event, Level};
        match fs::read(path) {
            Ok(data) => Some(BASE64_STANDARD.encode(data)),
            Err(e) => {
                event!(Level::ERROR, "加载文件失败: {}\nerr:{}", path, e);
                None
            }
        }
    }

    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn text(mut self, content: &str) -> UniMessage {
        self.messages.push(Message::Text(Text {
            text: String::from(content),
        }));
        self
    }

    pub fn texts(mut self, contents: &Vec<String>) -> UniMessage {
        for content in contents {
            self.messages.push(Message::Text(Text {
                text: String::from(content),
            }));
        }
        self
    }

    pub fn at(mut self, qq: String) -> UniMessage {
        self.messages.push(Message::At(At { qq: qq }));
        self
    }
    pub fn face(mut self, id: String) -> UniMessage {
        self.messages.push(Message::Face(Face { id: id }));
        self
    }
    pub fn image(mut self, image: FileType) -> UniMessage {
        match image {
            FileType::Url(u, t) => {
                self.messages.push(Message::Image(Image {
                    file: String::from("url"),
                    type_: None,
                    url: Some(u),
                    cache: Some(1),
                    proxy: None,
                    timeout: Some(t),
                }));
            }
            FileType::Path(f) => {
                if let Some(b64) = UniMessage::load_file_as_base64(&f) {
                    self.messages.push(Message::Image(Image {
                        file: String::from("base64://".to_owned() + &b64),
                        type_: None,
                        url: None,
                        cache: Some(1),
                        proxy: None,
                        timeout: None,
                    }));
                }
            }
            FileType::Base64(b64) => {
                self.messages.push(Message::Image(Image {
                    file: String::from("base64://".to_owned() + &b64),
                    type_: None,
                    url: None,
                    cache: Some(1),
                    proxy: None,
                    timeout: None,
                }));
            }
        }
        self
    }

    pub fn images(mut self, images: Vec<FileType>) -> UniMessage {
        for image in images {
            self = self.image(image);
        }
        self
    }

    /// record不可与其他消息类型组合发送！
    pub fn record(mut self, record: FileType) -> UniMessage {
        match record {
            FileType::Url(u, t) => {
                self.messages.push(Message::Record(Record {
                    file: String::from("url"),
                    magic: None,
                    url: Some(u),
                    cache: Some(1),
                    proxy: None,
                    timeout: Some(t),
                }));
            }
            FileType::Path(f) => {
                if let Some(b64) = UniMessage::load_file_as_base64(&f) {
                    self.messages.push(Message::Record(Record {
                        file: String::from("base64://".to_owned() + &b64),
                        magic: None,
                        url: None,
                        cache: Some(1),
                        proxy: None,
                        timeout: None,
                    }));
                }
            }
            FileType::Base64(b64) => {
                self.messages.push(Message::Record(Record {
                    file: String::from("base64://".to_owned() + &b64),
                    magic: None,
                    url: None,
                    cache: Some(1),
                    proxy: None,
                    timeout: None,
                }));
            }
        }
        self
    }
    pub fn build(self) -> Vec<Message> {
        self.messages
    }
}
