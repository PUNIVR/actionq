//use futures_util::{SinkExt, StreamExt};
//use serde::{Deserialize, Serialize};
//use std::net::SocketAddr;
//use tokio::net::{TcpListener, TcpStream};
//use tokio::sync::{broadcast, mpsc};
//use tokio_tungstenite::{accept_async, WebSocketStream};
//use tungstenite::protocol::Message;
//use glam::Vec2;
//
//use crate::common::{Request, RequestExerciseReps};
//use crate::{
//    pose::{PoseProxy, PoseEventSink},
//    session::{SessionProxy, SessionPoseData},
//};
//
//type ServerResult = Result<(), Box<dyn std::error::Error>>;
//
//#[derive(Debug, Clone, Copy, Serialize)]
//struct JsonKeypoint {
//    x: f32,
//    y: f32,
//}
//
//impl From<Vec2> for JsonKeypoint {
//    fn from(item: Vec2) -> Self {
//        Self { 
//            x: item.x, 
//            y: item.y 
//        }
//    }
//}
//
//#[derive(Debug, Serialize)]
//struct JsonPoseData {
//    keypoints: [JsonKeypoint; 20],
//}
//
//impl From<SessionPoseData> for JsonPoseData {
//    fn from(item: SessionPoseData) -> Self {
//        let mut keypoints: [JsonKeypoint; 20] = [JsonKeypoint{x:0.0,y:0.0}; 20];
//        for i in 0..item.keypoints.len() {
//            keypoints[i].x = item.keypoints[i].x;
//            keypoints[i].y = item.keypoints[i].y;
//        }
//
//        Self { keypoints }
//    }
//}
//
///// Possible responses from the client
//#[derive(Debug, Serialize)]
//#[serde(tag = "type")]
//pub enum Responses {
//    SessionConnect {
//        /// If true then the session was already in progress
//        already_in_progress: bool,
//    },
//    SessionEnd,
//    ExerciseStart,
//    ExerciseUpdate {
//        pose: JsonPoseData,
//    },
//    ExerciseEnd,
//    CloseAll,
//}
//
//async fn send_response(ws_stream: &mut WebSocketStream<TcpStream>, response: Responses) {
//    let json = serde_json::to_string(&response).unwrap();
//    ws_stream.send(json.into()).await.unwrap();
//}
//
//#[tracing::instrument(skip_all, fields(msg))]
//async fn handle_message(
//    ws_stream: &mut WebSocketStream<TcpStream>,
//    msg: Message,
//) -> Option<Request> {
//    if msg.is_text() {
//        // Try to deserialize command received
//        if let Ok(json) = msg.into_text() {
//            if let Ok(request) = serde_json::from_str::<Request>(&json) {
//                ws_stream.send("command received".into()).await.unwrap();
//                return Some(request);
//            }
//        } else {
//            ws_stream.send("command is not valid".into()).await.unwrap();
//        }
//    } else {
//        ws_stream.send("command is not text".into()).await.unwrap();
//    }
//    None
//}
//
///// Handles a single connection, sends messages to controller and sends data to client
//#[tracing::instrument(skip_all, fields(peer = %peer, connection_id))]
//async fn run_connection(
//    connection_id: usize,
//    kill_tx: broadcast::Sender<()>,
//    peer: SocketAddr,
//    stream: TcpStream,
//    session: SessionProxy,
//    pose: PoseProxy,
//) {
//    let mut ws_stream = accept_async(stream)
//        .await
//        .expect("unable to accept WebSocket connection");
//
//    tracing::info!("connection accepted");
//
//    // Create receiver for kill signal
//    let mut kill_rx = kill_tx.subscribe();
//
//    // Request a data channel connection
//    let mut data_receiver = session.connect_output_stream().await;
//
//    // Process all events
//    loop {
//        tokio::select! {
//
//            // Handle incomming requests from the client
//            client_packet = ws_stream.next() => {
//                let msg = client_packet.unwrap();
//                tracing::trace!("received message");
//
//                if let Some(request) = handle_message(&mut ws_stream, msg.unwrap()).await {
//                    match request {
//                        Request::SessionStart { exercises, save } => {
//                            tracing::info!("request: session start");
//                            session.session_start(exercises, save).await;
//                            send_response(&mut ws_stream, Responses::SessionConnect {
//                                already_in_progress: false, // NOTE: just for testing...
//                            }).await;
//                        }
//                        Request::SessionEnd => {
//                            tracing::info!("request: session end");
//                            session.session_end().await;
//                            send_response(&mut ws_stream, Responses::SessionEnd).await;
//                        },
//                        Request::ExerciseStart { exercise_id } => {
//                            tracing::info!("request: exercise start");
//                            session.exercise_start(exercise_id).await;
//                            send_response(&mut ws_stream, Responses::ExerciseStart).await;
//                        },
//                        Request::ExerciseEnd => {
//                            tracing::info!("request: exercise end");
//                            session.exercise_end().await;
//                            send_response(&mut ws_stream, Responses::ExerciseEnd).await;
//                        },
//                        Request::CloseAll => {
//                            tracing::info!("request: close all connections");
//                            kill_tx.send(()).unwrap();
//                            send_response(&mut ws_stream, Responses::CloseAll).await;
//                        }
//                    }
//                }
//            }
//
//            // Handle direct output data stream from the rest of the engine
//            engine_data = data_receiver.recv() => {
//                if let Ok(pose) = engine_data {
//                    // TODO: disabled for now, decide what to do here...
//                    //tracing::trace!("forward data");
//                    //send_response(&mut ws_stream, Responses::ExerciseUpdate {
//                    //    pose: pose.into()
//                    //}).await;
//                }
//            }
//
//            // TODO: Handle commands from the global connection controller
//            _ = kill_rx.recv() => {
//                tracing::info!("connection closed");
//                return;
//            }
//        }
//    }
//}
//
///// Accepts incomming connections and spawns handlers.
//#[tracing::instrument(skip_all)]
//pub async fn run_websocket_server(
//    addr: &str,
//    session_proxy: &SessionProxy,
//    pose_proxy: &PoseProxy,
//) -> ServerResult {
//    let listener = TcpListener::bind(addr).await?;
//
//    // Cancellation controller for all connections
//    let (kill_tx, kill_rx) = broadcast::channel(100);
//    drop(kill_rx);
//
//    tracing::info!("waiting connection");
//    let mut connection_id = 0;
//    while let Ok((stream, _)) = listener.accept().await {
//        let peer = stream.peer_addr()?;
//        tracing::info!("spawn connecion handler");
//        tokio::spawn(run_connection(
//            connection_id,
//            kill_tx.clone(),
//            peer,
//            stream,
//            session_proxy.clone(),
//            pose_proxy.clone(),
//        ));
//
//        connection_id += 1;
//    }
//    Ok(())
//}
