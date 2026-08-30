use crate::utils::{id_deserializer, option_id_deserializer};
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
    Dice(DiceData),

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
    Location(Location),

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
    /// 商城表情 (llonebot 扩展)
    #[serde(rename = "mface")]
    Mface(Mface),
    /// 文件 (llonebot 扩展)
    #[serde(rename = "file")]
    File(File),
    /// 键盘 (llonebot 扩展)
    #[serde(rename = "keyboard")]
    Keyboard(Keyboard),
    /// Markdown (llonebot 扩展)
    #[serde(rename = "markdown")]
    Markdown(Markdown),
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
    /// @某人 名称 (llonebot 扩展)
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiceData {
    /// 掷骰子结果
    #[serde(default, deserialize_with = "id_deserializer")]
    pub result: String,
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
pub struct Location {
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mface {
    /// 表情描述/摘要
    pub summary: String,
    /// 表情图片 URL
    pub url: String,
    /// 表情 ID
    pub emoji_id: String,
    /// 表情包 ID
    pub emoji_package_id: i64,
    /// 表情 key
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    /// 文件名
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
    pub rows: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Markdown {
    /// Markdown 内容
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
}
