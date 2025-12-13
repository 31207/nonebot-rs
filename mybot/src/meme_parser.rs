use crate::meme_info::MemeInfo;
use nonebot_rs::Message;
use nonebot_rs::{matcher::prelude::*, message::FileType, message::UniMessage};
use reqwest;
use tokio::sync::Mutex;
use tracing::{Level, event};
pub struct MemeParser {
    meme_infos: Mutex<Vec<MemeInfo>>,
}

#[async_trait]
impl Handler<MessageEvent> for MemeParser {
    async fn init(&self) {
        let mut meme_infos = self.meme_infos.lock().await;
        let resp = reqwest::get("http://127.0.0.1:2233/meme/infos").await;
        let result = match resp {
            Ok(resp) => resp.text().await,
            Err(err) => Err(err),
        };
        match result {
            Ok(data) => {
                let meme_info_tmp: Result<Vec<MemeInfo>, serde_json::Error> =
                    serde_json::from_str(data.as_str());
                match meme_info_tmp {
                    Ok(m) => {
                        *meme_infos = m;
                    }
                    Err(err) => {
                        event!(Level::ERROR, "读取meme info错误: {}", err);
                    }
                }
            }
            Err(err) => {
                event!(Level::ERROR, "读取meme info错误: {}", err);
            }
        }
    }

    on_message!(MessageEvent);

    async fn handle(&self, event: MessageEvent, matcher: Matcher<MessageEvent>) {
        let msg = event.get_message();

        let (texts, images) = parse(&msg, &matcher).await;
        let meme_infos = self.meme_infos.lock().await;
        if texts.is_empty() {
            return;
        }
        let meme_info = search_meme_by_keyword(texts[0].as_str(), &meme_infos);
        match meme_info {
            Some(meme_info) => match is_count_vaild(&meme_info, texts.len() - 1, images.len()) {
                Ok(_) => {
                    let make_result = make_meme(&meme_info, &texts[1..].to_vec(), &images).await;
                    match make_result {
                        Ok(base64_str) => {
                            matcher
                                .send(
                                    UniMessage::new()
                                        .image(FileType::Base64(base64_str))
                                        .build(),
                                )
                                .await;
                        }
                        Err(err) => {
                            matcher
                                .send_text(format!("制作meme失败: {}", err).as_str())
                                .await;
                        }
                    }
                }
                Err(err) => {
                    matcher.send_text(format!("{}", err).as_str()).await;
                }
            },
            None => return,
        }
    }
}

pub fn meme_parser() -> Matcher<MessageEvent> {
    Matcher::new(
        "meme_parser",
        MemeParser {
            meme_infos: Mutex::new(Vec::new()),
        },
    )
    .add_rule(rules::is_superuser())
}

/// 解析消息，提取文本和图片URL，包括第一层回复消息中的内容，丢弃其他类型消息
async fn parse(msg: &Vec<Message>, matcher: &Matcher<MessageEvent>) -> (Vec<String>, Vec<String>) {
    let mut texts: Vec<String> = Vec::new();
    let mut images: Vec<String> = Vec::new();

    for segment in msg.iter() {
        match segment {
            Message::Text(t) => {
                texts.append(
                    t.text
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .as_mut(),
                );
            }
            Message::Image(i) => {
                if let Some(url) = &i.url {
                    images.push(url.clone());
                }
            }
            Message::Reply(r) => {
                let message_id = r.id.parse::<i32>().unwrap();
                let replied_msg = matcher.get_msg(message_id).await;
                if let Some(replied_msg) = replied_msg {
                    match replied_msg {
                        nonebot_rs::api_resp::RespMessage::Group(g) => {
                            let replied_msg_content = g.message;
                            for segment in replied_msg_content.iter() {
                                match segment {
                                    Message::Text(t) => {
                                        texts.append(
                                            t.text
                                                .split_whitespace()
                                                .map(|s| s.to_string())
                                                .collect::<Vec<String>>()
                                                .as_mut(),
                                        );
                                    }
                                    Message::Image(i) => {
                                        if let Some(url) = &i.url {
                                            images.push(url.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        nonebot_rs::api_resp::RespMessage::Private(p) => {
                            let replied_msg_content = p.message;
                            for segment in replied_msg_content.iter() {
                                match segment {
                                    Message::Text(t) => {
                                        texts.append(
                                            t.text
                                                .split_whitespace()
                                                .map(|s| s.to_string())
                                                .collect::<Vec<String>>()
                                                .as_mut(),
                                        );
                                    }
                                    Message::Image(i) => {
                                        if let Some(url) = &i.url {
                                            images.push(url.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    (texts, images)
}

fn search_meme_by_keyword(keyword: &str, meme_infos: &Vec<MemeInfo>) -> Option<MemeInfo> {
    for meme_info in meme_infos.iter() {
        if meme_info.keywords.contains(&keyword.to_string()) {
            return Some(meme_info.clone());
        }
    }
    None
}

fn is_count_vaild(
    meme_info: &MemeInfo,
    text_count: usize,
    image_count: usize,
) -> Result<(), String> {
    let (min_texts, max_texts) = (meme_info.params.min_texts, meme_info.params.max_texts);
    let (min_images, max_images) = (meme_info.params.min_images, meme_info.params.max_images);
    if text_count >= min_texts
        && text_count <= max_texts
        && image_count >= min_images
        && image_count <= max_images
    {
        Ok(())
    } else {
        Err(format!(
            "文本或图片数量不符合要求: texts[{}, {}], images[{}, {}], 但是text: {}, image: {}",
            min_texts, max_texts, min_images, max_images, text_count, image_count
        ))
    }
}

async fn make_meme(
    meme_info: &MemeInfo,
    texts: &Vec<String>,
    images: &Vec<String>,
) -> Result<String, String> {
    let mut image_ids: Vec<String> = Vec::new();
    for image_url in images.iter() {
        let upload_result = crate::meme_api::upload_image(image_url).await;
        match upload_result {
            Ok(image_id) => {
                event!(Level::INFO, "Uploaded image ID: {}", image_id);
                image_ids.push(image_id);
            }
            Err(err) => {
                return Err(format!("Upload image failed: {}", err));
            }
        }
    }
    let meme_result = crate::meme_api::memes(
        &meme_info.key,
        texts.clone(),
        image_ids,
        serde_json::json!({}),
    )
    .await;
    match meme_result {
        Ok(image_id) => {
            event!(Level::INFO, "Meme created with image ID: {}", image_id);
            let base64_result = crate::meme_api::get_image_base64(&image_id).await;
            match base64_result {
                Ok(base64_str) => {
                    event!(
                        Level::INFO,
                        "Got base64 string of length: {}",
                        base64_str.len()
                    );
                    return Ok(base64_str);
                }
                Err(err) => {
                    return Err(format!("Get image base64 failed: {}", err));
                }
            }
        }
        Err(err) => {
            return Err(format!("Create meme failed: {}", err));
        }
    }
}
