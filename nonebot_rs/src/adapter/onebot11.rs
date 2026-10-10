//! OneBot 11 协议适配器
//!
//! 面向 OneBot 11 协议的主要部分进行宽松解析，忽略 llonebot/NapCat 等不同实现
//! 产生的 JSON 细节差异：
//!
//! - 实现特定字段（`raw_pb`/`message_format`/`message_seq`/`group_name` 等）缺失时使用默认值
//! - ID/数字类字段兼容数字与字符串两种表示
//! - 未知消息段保留为 [`crate::message::Message::Unknown`]，不丢弃整个事件
//! - `message` 支持消息段数组与 CQ 字符串两种格式，并与 `raw_message` 互相补全
//! - 未知事件类型丢弃并记录警告日志

use super::{Adapter, AdapterError};
use crate::api::Api;
use crate::api_resp::ApiResp;
use crate::config::AccessToken;
use crate::event::{Event, RecvItem};
use crate::message::{parse_cq_string, raw_message_from_segments, Message};
use colored::*;
use tracing::{event, Level};

/// OneBot 11 适配器名称
pub const ADAPTER_NAME: &str = "onebot_v11";

const KNOWN_POST_TYPES: [&str; 5] = ["message", "message_sent", "notice", "request", "meta_event"];

/// OneBot 11 协议适配器
#[derive(Debug, Default, Clone, Copy)]
pub struct OneBotV11Adapter;

impl Adapter for OneBotV11Adapter {
    fn name(&self) -> &'static str {
        ADAPTER_NAME
    }

    fn parse_frame(&self, raw: &str) -> Result<Vec<RecvItem>, AdapterError> {
        let value: serde_json::Value = serde_json::from_str(raw)
            .map_err(|e| AdapterError::malformed(ADAPTER_NAME, format!("JSON 解析失败: {e}")))?;

        if let Some(post_type) = value.get("post_type").and_then(|v| v.as_str()) {
            if !KNOWN_POST_TYPES.contains(&post_type) {
                return Err(AdapterError::unsupported(
                    ADAPTER_NAME,
                    format!("未知事件类型 post_type={post_type}"),
                ));
            }
            let mut event: Event = serde_json::from_value(value).map_err(|e| {
                AdapterError::decode(ADAPTER_NAME, format!("事件解析失败: {e}"))
            })?;
            normalize_event(&mut event);
            return Ok(vec![RecvItem::Event(event)]);
        }

        if value.get("echo").is_some()
            || value.get("status").is_some()
            || value.get("retcode").is_some()
        {
            let api_resp: ApiResp = serde_json::from_value(value).map_err(|e| {
                AdapterError::decode(ADAPTER_NAME, format!("Api 响应解析失败: {e}"))
            })?;
            return Ok(vec![RecvItem::ApiResp(api_resp)]);
        }

        Err(AdapterError::unsupported(
            ADAPTER_NAME,
            "既不是事件也不是 Api 响应",
        ))
    }

    fn encode_api(&self, api: &Api) -> Result<String, AdapterError> {
        serde_json::to_string(api)
            .map_err(|e| AdapterError::decode(ADAPTER_NAME, format!("Api 编码失败: {e}")))
    }

    fn handshake(
        &self,
        headers: &http::HeaderMap,
        access_token: &AccessToken,
    ) -> Result<String, String> {
        let get = |key: &str| {
            headers
                .get(key)
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string())
        };
        let bot_id = get("X-Self-ID").ok_or("缺少 X-Self-ID 请求头")?;
        let client_role = get("X-Client-Role").ok_or("缺少 X-Client-Role 请求头")?;
        if client_role != "Universal" {
            return Err(format!(
                "X-Client-Role 必须为 Universal，当前为 {client_role}"
            ));
        }
        let user_agent = get("User-Agent").unwrap_or_default();
        if !access_token.check_auth(&bot_id, get("Authorization")) {
            return Err("Access Token 校验失败".to_string());
        }
        event!(
            Level::INFO,
            "{} Client {} is connectted. The client type is {}",
            user_agent.bright_yellow(),
            bot_id.red(),
            client_role.bright_cyan()
        );
        Ok(bot_id)
    }
}

/// 归一化消息事件：`message` 数组与 `raw_message` 互相补全
fn normalize_event(event: &mut Event) {
    let message_event = match event {
        Event::Message(m) | Event::MessageSent(m) => m,
        _ => return,
    };
    match message_event {
        crate::event::MessageEvent::Private(p) => {
            normalize_message_fields(&mut p.message, &mut p.raw_message)
        }
        crate::event::MessageEvent::Group(g) => {
            normalize_message_fields(&mut g.message, &mut g.raw_message)
        }
    }
}

fn normalize_message_fields(message: &mut Vec<Message>, raw_message: &mut String) {
    if message.is_empty() && !raw_message.is_empty() {
        *message = parse_cq_string(raw_message);
    } else if raw_message.is_empty() && !message.is_empty() {
        *raw_message = raw_message_from_segments(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AccessToken;
    use crate::event::{MessageEvent, NoticeEvent};
    use std::collections::HashMap;

    fn parse_event(raw: &str) -> Event {
        let adapter = OneBotV11Adapter;
        let items = adapter.parse_frame(raw).unwrap();
        match items.into_iter().next().unwrap() {
            RecvItem::Event(event) => event,
            _ => panic!("expected event"),
        }
    }

    #[test]
    fn test_napcat_minimal_group_message() {
        let raw = r#"{
            "time": 1700000000,
            "self_id": 123,
            "post_type": "message",
            "message_type": "group",
            "message_id": "456",
            "user_id": 789,
            "group_id": 111,
            "message": [{"type":"text","data":{"text":"hi"}}]
        }"#;
        match parse_event(raw) {
            Event::Message(MessageEvent::Group(g)) => {
                assert_eq!(g.group_id, "111");
                assert_eq!(g.user_id, "789");
                assert_eq!(g.message_id, 456);
                assert_eq!(g.message_seq, 0);
                assert_eq!(g.message_format, "array");
                assert_eq!(g.group_name, "");
                assert_eq!(g.raw_message, "hi");
                assert!(matches!(&g.message[0], Message::Text(t) if t.text == "hi"));
            }
            _ => panic!("expected group message event"),
        }
    }

    #[test]
    fn test_message_as_cq_string() {
        let raw = r#"{
            "time": 1700000000,
            "self_id": "123",
            "post_type": "message",
            "message_type": "private",
            "message_id": 1,
            "user_id": 2,
            "message": "hello[CQ:at,qq=123]",
            "message_format": "string"
        }"#;
        match parse_event(raw) {
            Event::Message(MessageEvent::Private(p)) => {
                assert_eq!(p.message.len(), 2);
                assert!(matches!(&p.message[0], Message::Text(t) if t.text == "hello"));
                assert!(matches!(&p.message[1], Message::At(a) if a.qq == "123"));
                assert_eq!(p.raw_message, "hello[CQ:at,qq=123]");
            }
            _ => panic!("expected private message event"),
        }
    }

    #[test]
    fn test_unknown_segment_does_not_break_event() {
        let raw = r#"{
            "time": 1,
            "self_id": 1,
            "post_type": "message",
            "message_type": "group",
            "message_id": 1,
            "user_id": 2,
            "group_id": 3,
            "message": [{"type":"custom_seg","data":{"foo":"bar"}},{"type":"text","data":{"text":"ok"}}]
        }"#;
        match parse_event(raw) {
            Event::Message(MessageEvent::Group(g)) => {
                assert_eq!(g.message.len(), 2);
                assert!(matches!(&g.message[0], Message::Unknown(u) if u.type_ == "custom_seg"));
            }
            _ => panic!("expected group message event"),
        }
    }

    #[test]
    fn test_unknown_post_type() {
        let adapter = OneBotV11Adapter;
        let err = adapter
            .parse_frame(r#"{"post_type":"unknown_event","self_id":1}"#)
            .unwrap_err();
        assert_eq!(err.kind, super::super::AdapterErrorKind::Unsupported);
    }

    #[test]
    fn test_unknown_notice_type() {
        let adapter = OneBotV11Adapter;
        let err = adapter
            .parse_frame(r#"{"post_type":"notice","notice_type":"nope","self_id":1,"time":1}"#)
            .unwrap_err();
        assert_eq!(err.kind, super::super::AdapterErrorKind::Decode);
    }

    #[test]
    fn test_notice_lenient_fields() {
        let raw = r#"{
            "post_type": "notice",
            "notice_type": "group_ban",
            "time": 1,
            "self_id": 1,
            "group_id": 2,
            "user_id": 3,
            "sub_type": "ban"
        }"#;
        match parse_event(raw) {
            Event::Notice(NoticeEvent::GroupBan(b)) => {
                assert_eq!(b.operator_id, "");
                assert_eq!(b.duration, 0);
            }
            _ => panic!("expected group_ban notice"),
        }
    }

    #[test]
    fn test_api_resp() {
        let adapter = OneBotV11Adapter;
        let items = adapter
            .parse_frame(r#"{"status":"ok","retcode":0,"data":{"message_id":1},"echo":"e1"}"#)
            .unwrap();
        match &items[0] {
            RecvItem::ApiResp(resp) => {
                assert_eq!(resp.echo, "e1");
                assert_eq!(resp.wording, "");
            }
            _ => panic!("expected api resp"),
        }
    }

    #[test]
    fn test_malformed_json() {
        let adapter = OneBotV11Adapter;
        let err = adapter.parse_frame("not json").unwrap_err();
        assert_eq!(err.kind, super::super::AdapterErrorKind::Malformed);
    }

    #[test]
    fn test_handshake() {
        let adapter = OneBotV11Adapter;
        let access_token = AccessToken {
            global: String::new(),
            bots: HashMap::new(),
        };
        let mut headers = http::HeaderMap::new();
        headers.insert("X-Self-ID", "123".parse().unwrap());
        headers.insert("X-Client-Role", "Universal".parse().unwrap());
        headers.insert("User-Agent", "test".parse().unwrap());
        assert_eq!(adapter.handshake(&headers, &access_token).unwrap(), "123");

        let mut bad_headers = http::HeaderMap::new();
        bad_headers.insert("X-Self-ID", "123".parse().unwrap());
        bad_headers.insert("X-Client-Role", "Event".parse().unwrap());
        assert!(adapter.handshake(&bad_headers, &access_token).is_err());
    }
}
