use crate::utils::{
    flexible_string_deserializer, id_deserializer, lenient_i64_deserializer,
    option_id_deserializer, scalar_string_deserializer,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Onebot 协议消息定义
#[derive(Debug, Clone)]
pub enum Message {
    /// 纯文本
    Text(Text),

    /// QQ 表情
    Face(Face),

    /// 图片
    Image(Image),

    /// 语音
    Record(Record),

    /// 短视频
    Video(Video),

    /// @某人
    At(At),

    /// 猜拳魔法表情
    Rps(Rps),

    /// 掷骰子魔法表情
    Dice(DiceData),

    /// 窗口抖动（戳一戳）
    Shake(Shake),

    /// 戳一戳
    Poke(Poke),

    /// 匿名发消息
    Anonymous(Anonymous),

    /// 链接分享
    Share(Share),

    /// 推荐好友|群
    Contact(Contact),

    /// 位置
    Location(Location),

    /// 音乐分享
    Music(Music),

    /// 回复
    Reply(Reply),

    /// 合并转发
    Forward(Forward),

    /// 合并转发节点
    Node(Node),

    /// XML 消息
    Xml(Xml),

    /// JSON 消息
    Json(Json),
    /// 商城表情 (llonebot 扩展)
    Mface(Mface),
    /// 文件 (llonebot 扩展)
    File(File),
    /// 键盘 (llonebot 扩展)
    Keyboard(Keyboard),
    /// Markdown (llonebot 扩展)
    Markdown(Markdown),
    /// 未知/实现自定义消息段，保留原始数据
    Unknown(UnknownSegment),
}

/// 未知消息段
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnknownSegment {
    /// 消息段类型
    #[serde(rename = "type")]
    pub type_: String,
    /// 消息段原始数据
    pub data: Option<serde_json::Value>,
}

impl Message {
    /// 由消息段类型与 `data` 构造消息段，未知类型返回 [`Message::Unknown`]
    pub fn from_segment(type_: &str, data: serde_json::Value) -> Result<Self, String> {
        let data = if data.is_null() {
            serde_json::json!({})
        } else {
            data
        };
        let parsed = match type_ {
            "text" => serde_json::from_value(data).map(Message::Text),
            "face" => serde_json::from_value(data).map(Message::Face),
            "image" => serde_json::from_value(data).map(Message::Image),
            "record" => serde_json::from_value(data).map(Message::Record),
            "video" => serde_json::from_value(data).map(Message::Video),
            "at" => serde_json::from_value(data).map(Message::At),
            "rps" => serde_json::from_value(data).map(Message::Rps),
            "dice" => serde_json::from_value(data).map(Message::Dice),
            "shake" => serde_json::from_value(data).map(Message::Shake),
            "poke" => serde_json::from_value(data).map(Message::Poke),
            "anonymous" => serde_json::from_value(data).map(Message::Anonymous),
            "share" => serde_json::from_value(data).map(Message::Share),
            "contact" => serde_json::from_value(data).map(Message::Contact),
            "location" => serde_json::from_value(data).map(Message::Location),
            "music" => serde_json::from_value(data).map(Message::Music),
            "reply" => serde_json::from_value(data).map(Message::Reply),
            "forward" => serde_json::from_value(data).map(Message::Forward),
            "node" => serde_json::from_value(data).map(Message::Node),
            "xml" => serde_json::from_value(data).map(Message::Xml),
            "json" => serde_json::from_value(data).map(Message::Json),
            "mface" => serde_json::from_value(data).map(Message::Mface),
            "file" => serde_json::from_value(data).map(Message::File),
            "keyboard" => serde_json::from_value(data).map(Message::Keyboard),
            "markdown" => serde_json::from_value(data).map(Message::Markdown),
            _ => {
                return Ok(Message::Unknown(UnknownSegment {
                    type_: type_.to_string(),
                    data: Some(data),
                }))
            }
        };
        parsed.map_err(|e| format!("消息段 [{}] 解析失败: {}", type_, e))
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error as SerializeError;
        let (type_, data) = match self {
            Message::Text(v) => ("text", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Face(v) => ("face", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Image(v) => ("image", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Record(v) => ("record", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Video(v) => ("video", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::At(v) => ("at", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Rps(v) => ("rps", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Dice(v) => ("dice", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Shake(v) => ("shake", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Poke(v) => ("poke", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Anonymous(v) => {
                ("anonymous", serde_json::to_value(v).map_err(SerializeError::custom)?)
            }
            Message::Share(v) => ("share", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Contact(v) => ("contact", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Location(v) => ("location", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Music(v) => ("music", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Reply(v) => ("reply", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Forward(v) => ("forward", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Node(v) => ("node", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Xml(v) => ("xml", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Json(v) => ("json", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Mface(v) => ("mface", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::File(v) => ("file", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Keyboard(v) => ("keyboard", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Markdown(v) => ("markdown", serde_json::to_value(v).map_err(SerializeError::custom)?),
            Message::Unknown(u) => (
                u.type_.as_str(),
                u.data.clone().unwrap_or_else(|| serde_json::json!({})),
            ),
        };
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", type_)?;
        map.serialize_entry("data", &data)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawSegment {
            #[serde(rename = "type")]
            type_: String,
            #[serde(default)]
            data: serde_json::Value,
        }
        let segment = RawSegment::deserialize(deserializer)?;
        Message::from_segment(&segment.type_, segment.data).map_err(serde::de::Error::custom)
    }
}

/// 反序列化 `message` 字段，兼容消息段数组与 CQ 字符串两种格式
pub fn message_content_deserializer<'de, D>(deserializer: D) -> Result<Vec<Message>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(message_vec_from_value(value))
}

/// 反序列化可选的 `message` 字段，兼容消息段数组与 CQ 字符串两种格式
pub fn option_message_content_deserializer<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<Message>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    Ok(value.map(message_vec_from_value))
}

/// 将 `message` 字段的 JSON 值转换为消息段数组
pub fn message_vec_from_value(value: serde_json::Value) -> Vec<Message> {
    match value {
        serde_json::Value::String(raw) => parse_cq_string(&raw),
        serde_json::Value::Array(items) => items
            .into_iter()
            .filter_map(|item| match serde_json::from_value::<Message>(item) {
                Ok(message) => Some(message),
                Err(e) => {
                    tracing::warn!("丢弃无法解析的消息段: {}", e);
                    None
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn cq_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('[', "&#91;")
        .replace(']', "&#93;")
        .replace(',', "&#44;")
}

fn cq_unescape(s: &str) -> String {
    s.replace("&#91;", "[")
        .replace("&#93;", "]")
        .replace("&#44;", ",")
        .replace("&amp;", "&")
}

/// 解析 CQ 字符串为消息段数组，未知消息段保留为 [`Message::Unknown`]
pub fn parse_cq_string(raw: &str) -> Vec<Message> {
    let mut messages = Vec::new();
    let mut rest = raw;
    while let Some(start) = rest.find("[CQ:") {
        let text = &rest[..start];
        if !text.is_empty() {
            messages.push(Message::Text(Text {
                text: cq_unescape(text),
            }));
        }
        let Some(end) = rest[start..].find(']') else {
            messages.push(Message::Text(Text {
                text: cq_unescape(&rest[start..]),
            }));
            return messages;
        };
        let code = &rest[start + 4..start + end];
        let (type_, params) = match code.split_once(',') {
            Some((t, p)) => (t, p),
            None => (code, ""),
        };
        let mut data = serde_json::Map::new();
        for pair in params.split(',') {
            if pair.is_empty() {
                continue;
            }
            let (key, value) = match pair.split_once('=') {
                Some((k, v)) => (k, v),
                None => (pair, ""),
            };
            data.insert(
                key.to_string(),
                serde_json::Value::String(cq_unescape(value)),
            );
        }
        match Message::from_segment(type_, serde_json::Value::Object(data)) {
            Ok(message) => messages.push(message),
            Err(e) => {
                tracing::warn!("CQ 消息段解析失败: {}", e);
                messages.push(Message::Unknown(UnknownSegment {
                    type_: type_.to_string(),
                    data: None,
                }));
            }
        }
        rest = &rest[start + end + 1..];
    }
    if !rest.is_empty() {
        messages.push(Message::Text(Text {
            text: cq_unescape(rest),
        }));
    }
    messages
}

/// 将消息段数组转换为 CQ 字符串
pub fn raw_message_from_segments(messages: &[Message]) -> String {
    messages.iter().map(Message::to_cq_string).collect()
}

impl Message {
    /// 转换为 CQ 字符串（保留主要字段）
    pub fn to_cq_string(&self) -> String {
        fn param(key: &str, value: &str) -> String {
            format!("{}={}", key, cq_escape(value))
        }
        match self {
            Message::Text(v) => cq_escape(&v.text),
            Message::Face(v) => format!("[CQ:face,{}]", param("id", &v.id)),
            Message::Image(v) => format!("[CQ:image,{}]", param("file", &v.file)),
            Message::Record(v) => format!("[CQ:record,{}]", param("file", &v.file)),
            Message::Video(v) => format!("[CQ:video,{}]", param("file", &v.file)),
            Message::At(v) => format!("[CQ:at,{}]", param("qq", &v.qq)),
            Message::Rps(_) => "[CQ:rps]".to_string(),
            Message::Dice(v) => format!("[CQ:dice,{}]", param("result", &v.result)),
            Message::Shake(_) => "[CQ:shake]".to_string(),
            Message::Poke(v) => format!(
                "[CQ:poke,{},{}]",
                param("type", &v.type_),
                param("id", &v.id)
            ),
            Message::Anonymous(_) => "[CQ:anonymous]".to_string(),
            Message::Share(v) => format!(
                "[CQ:share,{},{}]",
                param("url", &v.url),
                param("title", &v.title)
            ),
            Message::Contact(v) => format!(
                "[CQ:contact,{},{}]",
                param("type", &v.type_),
                param("id", &v.id)
            ),
            Message::Location(v) => format!(
                "[CQ:location,{},{}]",
                param("lat", &v.lat),
                param("lon", &v.lon)
            ),
            Message::Music(v) => {
                let mut result = format!("[CQ:music,{}", param("type", &v.type_));
                if let Some(id) = &v.id {
                    result.push(',');
                    result.push_str(&param("id", id));
                }
                result.push(']');
                result
            }
            Message::Reply(v) => format!("[CQ:reply,{}]", param("id", &v.id)),
            Message::Forward(v) => format!("[CQ:forward,{}]", param("id", &v.id)),
            Message::Node(v) => {
                let mut result = String::from("[CQ:node");
                if let Some(id) = &v.id {
                    result.push(',');
                    result.push_str(&param("id", id));
                }
                if let Some(user_id) = &v.user_id {
                    result.push(',');
                    result.push_str(&param("user_id", user_id));
                }
                if let Some(nickname) = &v.nickname {
                    result.push(',');
                    result.push_str(&param("nickname", nickname));
                }
                result.push(']');
                result
            }
            Message::Xml(v) => format!("[CQ:xml,{}]", param("data", &v.data)),
            Message::Json(v) => format!("[CQ:json,{}]", param("data", &v.data)),
            Message::Mface(v) => format!(
                "[CQ:mface,{},{},{},{},{}]",
                param("summary", &v.summary),
                param("url", &v.url),
                param("emoji_id", &v.emoji_id),
                param("emoji_package_id", &v.emoji_package_id.to_string()),
                param("key", &v.key)
            ),
            Message::File(v) => format!("[CQ:file,{}]", param("file", &v.file)),
            Message::Keyboard(_) => "[CQ:keyboard]".to_string(),
            Message::Markdown(v) => format!("[CQ:markdown,{}]", param("content", &v.content)),
            Message::Unknown(v) => {
                let mut result = format!("[CQ:{}", v.type_);
                if let Some(serde_json::Value::Object(map)) = &v.data {
                    for (key, value) in map {
                        let value = match value {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        result.push(',');
                        result.push_str(&param(key, &value));
                    }
                }
                result.push(']');
                result
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Text {
    #[serde(default)]
    pub text: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Face {
    /// QQ 表情 ID
    #[serde(default, deserialize_with = "id_deserializer")]
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Record {
    /// 语音文件名
    #[serde(default)]
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
    #[serde(default)]
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
    #[serde(default)]
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
    #[serde(default, deserialize_with = "id_deserializer")]
    pub qq: String,
    /// @某人 名称 (llonebot 扩展)
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiceData {
    /// 掷骰子结果
    #[serde(default, deserialize_with = "id_deserializer")]
    pub result: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Rps {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Shake {}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Anonymous {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Poke {
    /// 类型
    #[serde(rename = "type", default, deserialize_with = "id_deserializer")]
    pub type_: String,
    /// ID
    #[serde(default, deserialize_with = "id_deserializer")]
    pub id: String,
    /// 表情名
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Share {
    /// URL
    #[serde(default)]
    pub url: String,
    /// 标题
    #[serde(default)]
    pub title: String,
    /// 内容描述
    pub content: Option<String>,
    /// 图片 URl
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Contact {
    /// 类型 qq|group
    #[serde(rename = "type", default)]
    pub type_: String,
    /// QQ号|群号
    #[serde(default, deserialize_with = "id_deserializer")]
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    /// 纬度
    #[serde(default, deserialize_with = "scalar_string_deserializer")]
    pub lat: String,
    /// 经度           
    #[serde(default, deserialize_with = "scalar_string_deserializer")]
    pub lon: String,
    /// 标题  
    pub title: Option<String>,
    /// 内容描述
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Music {
    /// 类型 qq|163|xm|custom
    #[serde(rename = "type", default)]
    pub type_: String,
    /// 歌曲 ID
    #[serde(default, deserialize_with = "option_id_deserializer")]
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
    #[serde(default, deserialize_with = "id_deserializer")]
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Forward {
    /// 合并转发 ID
    #[serde(default, deserialize_with = "id_deserializer")]
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    /// 转发的消息 ID
    #[serde(default, deserialize_with = "option_id_deserializer")]
    pub id: Option<String>,
    /// 发送者 QQ 号        
    #[serde(default, deserialize_with = "option_id_deserializer")]
    pub user_id: Option<String>,
    /// 发送者昵称   
    pub nickname: Option<String>,
    /// 消息内容
    #[serde(default, deserialize_with = "option_message_content_deserializer")]
    pub content: Option<Vec<Message>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Xml {
    /// 回复的消息 ID
    #[serde(default, deserialize_with = "flexible_string_deserializer")]
    pub data: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Json {
    /// 回复的消息 ID
    #[serde(default, deserialize_with = "flexible_string_deserializer")]
    pub data: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mface {
    /// 表情描述/摘要
    #[serde(default)]
    pub summary: String,
    /// 表情图片 URL
    #[serde(default)]
    pub url: String,
    /// 表情 ID
    #[serde(default, deserialize_with = "id_deserializer")]
    pub emoji_id: String,
    /// 表情包 ID
    #[serde(default, deserialize_with = "lenient_i64_deserializer")]
    pub emoji_package_id: i64,
    /// 表情 key
    #[serde(default)]
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    /// 文件名
    #[serde(default)]
    pub file: String,
    /// 文件 URL
    pub url: Option<String>,
    /// 文件 ID
    pub file_id: Option<String>,
    /// 本地路径
    pub path: Option<String>,
    /// 文件大小
    #[serde(default, deserialize_with = "option_id_deserializer")]
    pub file_size: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Keyboard {
    /// 键盘按钮行 (结构复杂, 保留原始 JSON)
    #[serde(default)]
    pub rows: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Markdown {
    /// Markdown 内容
    #[serde(default)]
    pub content: String,
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
        self.messages.push(Message::At(At { qq: qq, name: None }));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uni_message_text() {
        let msgs = UniMessage::new().text("hello").build();
        assert_eq!(msgs.len(), 1);
        match &msgs[0] {
            Message::Text(t) => assert_eq!(t.text, "hello"),
            _ => panic!("expected Text message"),
        }
    }

    #[test]
    fn test_uni_message_multiple() {
        let msgs = UniMessage::new()
            .text("a")
            .text("b")
            .at("123".to_string())
            .build();
        assert_eq!(msgs.len(), 3);
        match &msgs[0] {
            Message::Text(t) => assert_eq!(t.text, "a"),
            _ => panic!("expected Text 'a'"),
        }
        match &msgs[1] {
            Message::Text(t) => assert_eq!(t.text, "b"),
            _ => panic!("expected Text 'b'"),
        }
        match &msgs[2] {
            Message::At(a) => assert_eq!(a.qq, "123"),
            _ => panic!("expected At '123'"),
        }
    }

    #[test]
    fn test_uni_message_image_base64() {
        let msgs = UniMessage::new()
            .image(FileType::Base64("abc".to_string()))
            .build();
        assert_eq!(msgs.len(), 1);
        match &msgs[0] {
            Message::Image(img) => assert_eq!(img.file, "base64://abc"),
            _ => panic!("expected Image message"),
        }
    }

    #[test]
    fn test_text_message_serde() {
        let msg = Message::Text(Text {
            text: "hello".to_string(),
        });
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"text","data":{"text":"hello"}}"#);
    }

    #[test]
    fn test_empty_data_segment_serde() {
        let rps: Message = serde_json::from_str(r#"{"type":"rps","data":{}}"#).unwrap();
        assert!(matches!(rps, Message::Rps(_)));
        assert_eq!(
            serde_json::to_string(&rps).unwrap(),
            r#"{"type":"rps","data":{}}"#
        );

        let shake: Message = serde_json::from_str(r#"{"type":"shake","data":{}}"#).unwrap();
        assert!(matches!(shake, Message::Shake(_)));
        assert_eq!(
            serde_json::to_string(&shake).unwrap(),
            r#"{"type":"shake","data":{}}"#
        );

        let anonymous: Message =
            serde_json::from_str(r#"{"type":"anonymous","data":{}}"#).unwrap();
        assert!(matches!(anonymous, Message::Anonymous(_)));
        assert_eq!(
            serde_json::to_string(&anonymous).unwrap(),
            r#"{"type":"anonymous","data":{}}"#
        );
    }

    #[test]
    fn test_missing_data_segment_serde() {
        let shake: Message = serde_json::from_str(r#"{"type":"shake"}"#).unwrap();
        assert!(matches!(shake, Message::Shake(_)));
    }

    #[test]
    fn test_unknown_segment_serde() {
        let msg: Message =
            serde_json::from_str(r#"{"type":"custom_seg","data":{"foo":1}}"#).unwrap();
        match &msg {
            Message::Unknown(u) => {
                assert_eq!(u.type_, "custom_seg");
                assert_eq!(u.data.as_ref().unwrap()["foo"], 1);
            }
            _ => panic!("expected Unknown message"),
        }
        assert_eq!(
            serde_json::to_string(&msg).unwrap(),
            r#"{"type":"custom_seg","data":{"foo":1}}"#
        );
    }

    #[test]
    fn test_numeric_fields_lenient() {
        let msg: Message = serde_json::from_str(r#"{"type":"at","data":{"qq":123}}"#).unwrap();
        assert!(matches!(&msg, Message::At(a) if a.qq == "123"));

        let msg: Message = serde_json::from_str(r#"{"type":"reply","data":{"id":456}}"#).unwrap();
        assert!(matches!(&msg, Message::Reply(r) if r.id == "456"));

        let msg: Message =
            serde_json::from_str(r#"{"type":"mface","data":{"emoji_package_id":"244959"}}"#)
                .unwrap();
        assert!(matches!(&msg, Message::Mface(m) if m.emoji_package_id == 244959));
    }

    #[test]
    fn test_cq_string_parse() {
        let messages = parse_cq_string("hello[CQ:at,qq=123] world[CQ:face,id=14]");
        assert_eq!(messages.len(), 4);
        assert!(matches!(&messages[0], Message::Text(t) if t.text == "hello"));
        assert!(matches!(&messages[1], Message::At(a) if a.qq == "123"));
        assert!(matches!(&messages[2], Message::Text(t) if t.text == " world"));
        assert!(matches!(&messages[3], Message::Face(f) if f.id == "14"));
    }

    #[test]
    fn test_cq_string_escape() {
        let messages = parse_cq_string("&#91;CQ:at,qq=1&#93;");
        assert_eq!(messages.len(), 1);
        assert!(matches!(&messages[0], Message::Text(t) if t.text == "[CQ:at,qq=1]"));

        let messages = parse_cq_string("[CQ:at,qq=1,name=a&#44;b]");
        assert!(matches!(&messages[0], Message::At(a) if a.name.as_deref() == Some("a,b")));
    }

    #[test]
    fn test_cq_string_unknown_segment() {
        let messages = parse_cq_string("[CQ:custom_seg,foo=bar]");
        match &messages[0] {
            Message::Unknown(u) => {
                assert_eq!(u.type_, "custom_seg");
                assert_eq!(u.data.as_ref().unwrap()["foo"], "bar");
            }
            _ => panic!("expected Unknown message"),
        }
    }

    #[test]
    fn test_raw_message_from_segments() {
        let segments = vec![
            Message::Text(Text {
                text: "hi".to_string(),
            }),
            Message::At(At {
                qq: "123".to_string(),
                name: None,
            }),
        ];
        assert_eq!(raw_message_from_segments(&segments), "hi[CQ:at,qq=123]");
    }

    #[test]
    fn test_message_content_deserializer_cq() {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(deserialize_with = "message_content_deserializer")]
            message: Vec<Message>,
        }
        let wrapper: Wrapper = serde_json::from_str(r#"{"message":"hi[CQ:at,qq=1]"}"#).unwrap();
        assert_eq!(wrapper.message.len(), 2);
        assert!(matches!(&wrapper.message[1], Message::At(a) if a.qq == "1"));
    }

    #[test]
    fn test_node_content_cq_string() {
        let msg: Message = serde_json::from_str(
            r#"{"type":"node","data":{"user_id":1,"nickname":"a","content":"hi[CQ:at,qq=1]"}}"#,
        )
        .unwrap();
        match &msg {
            Message::Node(n) => {
                let content = n.content.as_ref().unwrap();
                assert_eq!(content.len(), 2);
                assert!(matches!(&content[1], Message::At(a) if a.qq == "1"));
            }
            _ => panic!("expected node"),
        }
    }

    #[test]
    fn test_json_segment_data_object() {
        let msg: Message =
            serde_json::from_str(r#"{"type":"json","data":{"data":{"app":"x"}}}"#).unwrap();
        match &msg {
            Message::Json(j) => assert!(j.data.contains("app")),
            _ => panic!("expected json"),
        }
    }
}
