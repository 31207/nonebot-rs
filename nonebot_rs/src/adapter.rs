//! 协议适配器
//!
//! 不同消息源（llonebot/NapCat 等 OneBot 实现，以及未来的 QQ 官方 Bot、Telegram 等）
//! 的 JSON 细节略有不同。适配器负责将各消息源的原始帧归一化为内部统一事件模型
//! （即本 crate 的 [`crate::event`] / [`crate::message`] 类型），
//! 并负责出站 [`Api`] 到线上 JSON 的编码。

pub mod onebot11;

use crate::api::Api;
use crate::config::AccessToken;
use crate::event::RecvItem;

/// 适配器错误种类，用于决定丢弃帧时的日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterErrorKind {
    /// JSON 语法错误，无法解析为 JSON
    Malformed,
    /// 合法 JSON 但事件/响应类型不受支持
    Unsupported,
    /// 已识别类型但字段解析失败
    Decode,
}

/// 适配器错误
#[derive(Debug, Clone)]
pub struct AdapterError {
    /// 错误种类
    pub kind: AdapterErrorKind,
    /// 所属适配器名称
    pub adapter: &'static str,
    /// 错误详情
    pub detail: String,
}

impl AdapterError {
    /// 构造 JSON 语法错误
    pub fn malformed(adapter: &'static str, detail: impl Into<String>) -> Self {
        AdapterError {
            kind: AdapterErrorKind::Malformed,
            adapter,
            detail: detail.into(),
        }
    }

    /// 构造不受支持类型错误
    pub fn unsupported(adapter: &'static str, detail: impl Into<String>) -> Self {
        AdapterError {
            kind: AdapterErrorKind::Unsupported,
            adapter,
            detail: detail.into(),
        }
    }

    /// 构造字段解析错误
    pub fn decode(adapter: &'static str, detail: impl Into<String>) -> Self {
        AdapterError {
            kind: AdapterErrorKind::Decode,
            adapter,
            detail: detail.into(),
        }
    }
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.kind {
            AdapterErrorKind::Malformed => "JSON 语法错误",
            AdapterErrorKind::Unsupported => "不支持的消息类型",
            AdapterErrorKind::Decode => "消息解析失败",
        };
        write!(f, "适配器 [{}] {}: {}", self.adapter, kind, self.detail)
    }
}

impl std::error::Error for AdapterError {}

/// 协议适配器
///
/// 实现该 trait 即可接入新的消息源：入站把原始帧解析为内部统一事件，
/// 出站把内部 [`Api`] 编码为该协议的线上 JSON。
pub trait Adapter: Send + Sync + std::fmt::Debug {
    /// 适配器名称，与配置文件中 bot 的 `adapter` 字段对应
    fn name(&self) -> &'static str;

    /// 解析收到的原始文本帧为内部事件/响应项（单帧可产生多项，便于批量协议扩展）
    fn parse_frame(&self, raw: &str) -> Result<Vec<RecvItem>, AdapterError>;

    /// 将内部 Api 编码为协议线上的 JSON 字符串（出站 hook）
    fn encode_api(&self, api: &Api) -> Result<String, AdapterError> {
        serde_json::to_string(api)
            .map_err(|e| AdapterError::decode(self.name(), format!("Api 编码失败: {e}")))
    }

    /// 反向 WebSocket 握手校验，成功时返回 bot_id
    ///
    /// 默认拒绝：不支持反向 WebSocket 的协议无需实现。
    fn handshake(
        &self,
        headers: &http::HeaderMap,
        access_token: &AccessToken,
    ) -> Result<String, String> {
        let _ = (headers, access_token);
        Err(format!("适配器 [{}] 不支持反向 WebSocket 握手", self.name()))
    }
}
