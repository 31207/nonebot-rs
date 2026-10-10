use super::utils::handler_web_socket;
use crate::adapter::Adapter;
use crate::{ActionSender, EventSender};
use colored::*;
use http::Response as HttpResponse;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tracing::{event, Level};

/// start Reverse WebSocket Server
pub async fn run(
    host: std::net::Ipv4Addr,
    port: u16,
    event_sender: EventSender,
    action_sender: ActionSender,
    access_token: crate::config::AccessToken,
    adapter: Arc<dyn Adapter>,
) {
    // bind address to start Tcp server
    let try_socket = TcpListener::bind(std::net::SocketAddrV4::new(host, port)).await;
    let listener = try_socket.expect("Socket Bind fail");
    event!(Level::INFO, "Serveing at -> ws://{}:{}/ws", host, port);

    // lopp wait for connect
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                event!(Level::TRACE, "Get a TCP connect");
                tokio::spawn(accept_connection(
                    stream,
                    event_sender.clone(),
                    action_sender.clone(),
                    access_token.clone(),
                    adapter.clone(),
                ));
            }
            Err(e) => event!(Level::WARN, "TCP connect error {}", e),
        }
    }
}

/// handle a income tcp connect
async fn accept_connection(
    stream: TcpStream,
    event_sender: EventSender,
    action_sender: ActionSender,
    access_token: crate::config::AccessToken,
    adapter: Arc<dyn Adapter>,
) {
    // check peer address
    stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    let mut output_bot_id = String::new();

    // callback to check headers && get bot_id, 握手规则交由适配器处理
    let handshake_adapter = adapter.clone();
    let callback =
        |req: &Request, resp: Response| -> Result<Response, HttpResponse<Option<String>>> {
            match handshake_adapter.handshake(req.headers(), &access_token) {
                Ok(bot_id) => {
                    output_bot_id = bot_id;
                    Ok(resp)
                }
                Err(e) => {
                    event!(Level::WARN, "WebSocket handshake rejected: {}", e.red());
                    Err(HttpResponse::new(None))
                }
            }
        };

    // Upgrade TcpStream to WebSocketStream
    let ws_stream = tokio_tungstenite::accept_hdr_async(stream, callback)
        .await
        .expect("TcpStream handshake fail");

    // build channel
    let (sender, receiver) = mpsc::channel(32);
    let (apiresp_watch_sender, api_resp_watcher) = watch::channel(crate::api_resp::ApiResp {
        status: "init".to_string(),
        retcode: 0,
        data: crate::api_resp::RespData::None,
        echo: "".to_string(),
        message: "".to_string(),
        wording: "".to_string(),
    });

    // add bot to Nonebot
    let _ = action_sender
        .send(crate::Action::AddBot {
            bot_id: output_bot_id.clone(),
            api_sender: sender,
            action_sender: action_sender.clone(),
            api_resp_watcher: api_resp_watcher,
        })
        .await;

    // handle WebSocketStream
    handler_web_socket(
        ws_stream,
        event_sender,
        action_sender,
        apiresp_watch_sender,
        receiver,
        output_bot_id,
        adapter,
    )
    .await;
}
