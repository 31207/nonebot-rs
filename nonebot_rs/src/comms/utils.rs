use crate::adapter::{Adapter, AdapterErrorKind};
use crate::{event::Event, ActionSender, EventSender};
use colored::*;
use futures_util::{stream::SplitStream, SinkExt, StreamExt};
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::{broadcast, watch},
};
use tokio_tungstenite::{tungstenite::Message as TuMessage, WebSocketStream};
use tracing::{event, Level};

pub async fn handler_web_socket(
    socket: WebSocketStream<TcpStream>,
    event_sender: EventSender,
    action_sender: ActionSender,
    apiresp_watch_sender: tokio::sync::watch::Sender<crate::ApiResp>,
    mut api_receiver: tokio::sync::mpsc::Receiver<crate::ApiChannelItem>,
    bot_id: String,
    adapter: Arc<dyn Adapter>,
) {
    // 将 websocket 接收流与发送流分离
    let (mut sink, mut stream) = socket.split();
    // 接收消息
    let another_event_sender = event_sender.clone();
    let income_adapter = adapter.clone();
    let income = async move {
        loop {
            let r = stream_recv(
                stream,
                &another_event_sender,
                &action_sender,
                &apiresp_watch_sender,
                bot_id.clone(),
                income_adapter.clone(),
            )
            .await;
            if let Some(s) = r {
                stream = s;
            } else {
                return;
            }
        }
    };
    // 发送消息
    let outcome = async move {
        while let Some(data) = api_receiver.recv().await {
            match data {
                // Onebot Api
                crate::ApiChannelItem::Api(api) => {
                    match adapter.encode_api(&api) {
                        Ok(json_string) => {
                            //println!("-> {}",json_string);
                            sink.send(TuMessage::text(json_string)).await.ok();
                        }
                        Err(e) => {
                            event!(
                                Level::ERROR,
                                "适配器 [{}] 编码 Api 失败: {}",
                                adapter.name(),
                                e
                            );
                        }
                    }
                }
                // temp Matcher event
                crate::ApiChannelItem::MessageEvent(_) => {
                    event!(
                        Level::WARN,
                        "{}",
                        "WedSocket接受端接收到错误Event消息".bright_red()
                    );
                }
                // temp Matcher Timeout
                crate::ApiChannelItem::TimeOut => {
                    event!(
                        Level::WARN,
                        "{}",
                        "WedSocket接受端接收到错误TimeOut消息".bright_red()
                    );
                } // 忽视 event 该 receiver 永不应该收到 event
            }
        }
    };
    tokio::spawn(income);
    outcome.await;
}

async fn stream_recv(
    stream: SplitStream<WebSocketStream<TcpStream>>,
    event_sender: &EventSender,
    action_sender: &ActionSender,
    apiresp_watch_sender: &watch::Sender<crate::api_resp::ApiResp>,
    bot_id: String,
    adapter: Arc<dyn Adapter>,
) -> Option<SplitStream<WebSocketStream<TcpStream>>> {
    let (msg, next_stream) = stream.into_future().await;
    // 仅文本/二进制帧承载 OneBot 消息，其余控制帧直接跳过
    let raw = match msg {
        Some(Ok(TuMessage::Text(text))) => text,
        Some(Ok(TuMessage::Binary(bytes))) => match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => {
                event!(Level::WARN, "Bot [{}] 收到非 UTF-8 二进制帧，已忽略", bot_id.red());
                return Some(next_stream);
            }
        },
        Some(Ok(TuMessage::Ping(_))) | Some(Ok(TuMessage::Pong(_))) => {
            return Some(next_stream);
        }
        Some(Ok(_)) | None => {
            disconnect(action_sender, &bot_id).await;
            return None;
        }
        Some(Err(_)) => {
            disconnect(action_sender, &bot_id).await;
            return None;
        }
    };

    use crate::event::RecvItem;
    match adapter.parse_frame(&raw) {
        Ok(items) => {
            for item in items {
                match item {
                    RecvItem::Event(event) => send_event(event_sender, event).await,
                    RecvItem::ApiResp(api_resp) => {
                        let _ = apiresp_watch_sender.send(api_resp);
                    }
                }
            }
        }
        Err(e) => {
            match e.kind {
                AdapterErrorKind::Malformed => {
                    event!(Level::ERROR, "Discard message: {}\nRaw: {}", e, raw)
                }
                AdapterErrorKind::Unsupported | AdapterErrorKind::Decode => {
                    event!(Level::WARN, "Discard message: {}\nRaw: {}", e, raw)
                }
            }
            if crate::config::debug_enabled() {
                use std::io::Write;
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("error_msg.txt")
                    .and_then(|mut f| writeln!(f, "{}", raw))
                    .ok();
            }
        }
    }
    Some(next_stream)
}

async fn disconnect(action_sender: &ActionSender, bot_id: &str) {
    event!(Level::WARN, "Bot [{}] disconnect", bot_id.red());
    action_sender
        .send(crate::Action::RemoveBot {
            bot_id: bot_id.to_string(),
        })
        .await
        .ok();
}

pub async fn send_event(sender: &broadcast::Sender<Event>, e: Event) {
    match sender.send(e) {
        Ok(_) => (),
        Err(broadcast::error::SendError(_)) => {
            event!(Level::ERROR, "EventChannel is full out of cache!");
        }
    }
}
