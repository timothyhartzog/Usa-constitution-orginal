use actix_web::{web, HttpRequest, HttpResponse, Error};
use futures::StreamExt;
use std::time::{Duration, Instant};
use crate::state::AppState;
use std::sync::Arc;
use serde::Serialize;
use log::info;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Serialize, Clone, Debug)]
pub struct IndexUpdateEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub chunks_added: u32,
    pub timestamp: i64,
}

pub async fn ws_handler(
    req: HttpRequest,
    body: web::Payload,
    state: web::Data<Arc<AppState>>,
) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let mut rx = state.subscribe();

    actix_web::rt::spawn(async move {
        let mut last_heartbeat = Instant::now();
        let mut heartbeat_interval = tokio::time::interval(HEARTBEAT_INTERVAL);
        
        loop {
            tokio::select! {
                // Handle heartbeat
                _ = heartbeat_interval.tick() => {
                    if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                        info!("Websocket client timed out");
                        let _ = session.close(None).await;
                        break;
                    }
                    if session.ping(b"").await.is_err() {
                        break;
                    }
                }
                
                // Handle incoming messages
                Some(Ok(msg)) = msg_stream.next() => {
                    match msg {
                        actix_ws::Message::Ping(bytes) => {
                            last_heartbeat = Instant::now();
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        actix_ws::Message::Pong(_) => {
                            last_heartbeat = Instant::now();
                        }
                        actix_ws::Message::Text(_) => {
                            last_heartbeat = Instant::now();
                            // Optional: handle client messages (like subscribe/unsubscribe)
                        }
                        actix_ws::Message::Close(reason) => {
                            let _ = session.close(reason).await;
                            break;
                        }
                        _ => {}
                    }
                }
                
                // Handle broadcast messages
                Ok(event) = rx.recv() => {
                    if let Ok(json) = serde_json::to_string(&event) {
                        if session.text(json).await.is_err() {
                            break;
                        }
                    }
                }
                
                else => break,
            }
        }
    });

    Ok(response)
}
