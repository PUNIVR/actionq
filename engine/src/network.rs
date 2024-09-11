use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

use crate::{
    hpe::{EngineProxy, PoseEventSink},
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
    //TODO: Start an exercise evaluation
    //ExerciseStart,

    //TODO: Stop the current exercise evaluation in progress
    //ExerciseEnd,
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
}

/// Handles a single connection, sends messages to controller and sends data to client
async fn run_connection(
    peer: SocketAddr,
    stream: TcpStream,
    session_proxy: SessionProxy,
    engine_proxy: EngineProxy,
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
                            let response = Responses::SessionConnect {
                                already_in_progress: is_controller, // NOTE: just for testing...
                            };

                            let json = serde_json::to_string(&response).unwrap();
                            ws_stream.send(json.into()).await.unwrap();
                        }
                        Requests::SessionEnd => {
                            let response = Responses::SessionEnd;
                            let json = serde_json::to_string(&response).unwrap();
                            ws_stream.send(json.into()).await.unwrap();
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
    engine_proxy: &EngineProxy,
) -> ServerResult {
    let listener = TcpListener::bind(addr).await?;

    println!("WebSocket - waiting connection");
    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr()?;
        tokio::spawn(run_connection(
            peer,
            stream,
            session_proxy.clone(),
            engine_proxy.clone(),
        ));
    }
    Ok(())
}
