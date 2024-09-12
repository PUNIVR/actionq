use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream};

use crate::{
    pose::{PoseProxy, PoseEventSink},
    session::SessionProxy,
};

type ServerResult = Result<(), Box<dyn std::error::Error>>;

/// Possible requests from the client
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Requests {
    /// Starts a new session, if one is already in progress then connect to that
    /// session without starting a new one
    SessionStart {
        /// If true this connection can command the session execution
        is_controller: bool,
    },
    /// End the current session in progress
    SessionEnd,
    /// Start an exercise evaluation
    ExerciseStart {
        exercise_id: usize
    },
    /// Stop the current exercise evaluation in progress
    ExerciseEnd,
}

/// Possible responses from the client
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Responses {
    SessionConnect {
        /// If true then the session was already in progress
        already_in_progress: bool,
    },
    SessionEnd,
    ExerciseStart,
    ExerciseEnd,
}

async fn send_response(ws_stream: &mut WebSocketStream<TcpStream>, response: Responses) {
    let json = serde_json::to_string(&response).unwrap();
    ws_stream.send(json.into()).await.unwrap();
}

/// Handles a single connection, sends messages to controller and sends data to client
async fn run_connection(
    peer: SocketAddr,
    stream: TcpStream,
    session_proxy: SessionProxy,
    pose_proxy: PoseProxy,
) {
    let mut ws_stream = accept_async(stream)
        .await
        .expect("unable to accept WebSocket connection");

    println!("Connection - accepted");

    // From here we can comunicate, handle incomming messages and send HPE data
    while let Some(msg) = ws_stream.next().await {
        let msg = msg.unwrap();
        if msg.is_text() {
            // Try to deserialize command received
            if let Ok(json) = msg.into_text() {
                if let Ok(request) = serde_json::from_str::<Requests>(&json) {
                    println!("{:?}", request);
                    ws_stream.send("command received".into()).await.unwrap();

                    match request {
                        Requests::SessionStart { is_controller } => {
                            session_proxy.session_start().await;
                            
                            send_response(&mut ws_stream, Responses::SessionConnect {
                                already_in_progress: is_controller, // NOTE: just for testing...
                            }).await;
                        }
                        Requests::SessionEnd => {
                            session_proxy.session_end().await;
                            send_response(&mut ws_stream, Responses::SessionEnd).await;
                        },
                        Requests::ExerciseStart { exercise_id } => {
                            session_proxy.exercise_start(exercise_id).await;
                            send_response(&mut ws_stream, Responses::ExerciseStart).await;
                        },
                        Requests::ExerciseEnd => {
                            session_proxy.exercise_end().await;
                            send_response(&mut ws_stream, Responses::ExerciseEnd).await;
                        }
                    }
                } else {
                    ws_stream.send("command is not valid".into()).await.unwrap();
                }
            } else {
                ws_stream.send("command is not text".into()).await.unwrap();
            }
        }
    }

    println!("Connection - closed");
}

/// Accepts incomming connections and spawns handlers.
pub async fn run_websocket_server(
    addr: &str,
    session_proxy: &SessionProxy,
    pose_proxy: &PoseProxy,
) -> ServerResult {
    let listener = TcpListener::bind(addr).await?;

    println!("WebSocket - waiting connection");
    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr()?;
        tokio::spawn(run_connection(
            peer,
            stream,
            session_proxy.clone(),
            pose_proxy.clone(),
        ));
    }
    Ok(())
}
