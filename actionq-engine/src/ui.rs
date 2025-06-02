#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::net::{TcpStream, TcpListener};
use tungstenite::{accept, WebSocket};
use tokio_tungstenite::{accept_async};
use futures_util::StreamExt;
use futures_util::SinkExt;

use actionq_common::*;
use actionq_motion::{LuaExercise, StateEvent, StateOutput, StateWarning, Widget, Metadata};

/// Contains all commands understood by the ui
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    SessionStart {
        /// Number of exercises
        exercises_count: u32,
        /// Ids of the exercises in the session
        exercise_ids: Vec<String>,
        /// Resolution of the screen
        resolution: (u32, u32),
        /// Framerate of the camera
        frame_rate: u32,
    },

    ExerciseStart {
        /// Id of the current exercise
        exercise_id: String,
        /// Repetitions number to reach
        repetitions_target: u32,
    },

    ExerciseUpdate {
        /// FSM metadata
        metadata: Option<Metadata>,
        /// Current 2D skeleton
        skeleton: SkeletonMap2D,
        /// Current number of repetitions
        repetitions: u32,
        /// Current framebuffer
        frame: Vec<u8>,
    },

    ExerciseEnd,
    SessionEnd,
}

#[derive(Debug)]
pub struct UiProxy(pub Sender<Command>);
impl UiProxy {
    pub async fn send(&self, cmd: Command) {
        self.0.send(cmd).await.unwrap();
    }
}

// Compress frame into jpeg
fn compress_frame(bytes_abgr: &[u8], w: usize, h: usize) -> Vec<u8> {
    let image = turbojpeg::Image {
        format: turbojpeg::PixelFormat::BGRA,
        pixels: bytes_abgr,
        pitch: w * 4,
        height: h,
        width: w,
    };  

    let jpeg = turbojpeg::compress(image.as_deref(), 75, turbojpeg::Subsamp::Sub2x2)
        .expect("unable to compress frame");

    Vec::from(jpeg.as_ref())
}

// URL of the http server
const UI_URL: &str = "127.0.0.1";

struct UiClient {}
impl UiClient {
    #[tracing::instrument(skip_all)]
    pub async fn run_ui_client(self, mut rx: Receiver<Command>) {
        
        // Create TCP server for the ui client
        let listener = TcpListener::bind(&format!("{}:9090", UI_URL)).await
            .expect("unable to create ui server");
           
        tracing::info!("TCP listener ready");

        // Spawn firefox in kiosk mode
        let kiosk = std::process::Command::new("firefox")
            .args(["--kiosk", &format!("http://{}:8080", UI_URL)])
            .spawn().expect("unable to spawn ui kiosk");

        tracing::info!("kiosk spawned");

        // Accept kiosk TCP connection
        let (stream, _) = listener.accept().await
            .expect("unable to connect to kiosk");

        tracing::info!("kiosk connected to TCP");

        // Upgrade to WebSocket connection
        let stream = accept_async(stream).await
            .expect("unable to upgrade kiosk connection from TCP to WebSocket");

        tracing::info!("kiosk connected to WebSocket");

        // Get new messages if available
        let (mut ui_write, _ui_read) = stream.split();
        while let Some(cmd) = rx.recv().await {
            tracing::info!("forwarding ui command...");

            // Compress video frame
            let cmd = match cmd {
                Command::ExerciseUpdate { metadata, skeleton, repetitions, frame } => Command::ExerciseUpdate {
                    metadata, skeleton, repetitions,
                    frame: compress_frame(&frame, 1280, 720) // TODO: make resolution dynamic
                },
                _ => cmd,
            };

            // Forward to websocket
            let cmd = serde_json::to_string(&cmd).expect("unable to serialize ui command");
            ui_write.send(tungstenite::Message::Text(cmd)).await
                .expect("unable to forward command to kiosk");
        }
    }
}

pub fn run_ui_client(rx: Receiver<Command>) {
    let client = UiClient { };
    tokio::spawn(client.run_ui_client(rx));
}
