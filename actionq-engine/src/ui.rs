#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{Receiver, Sender};

use actionq_common::*;
use actionq_motion::{LuaExercise, StateEvent, StateOutput, StateWarning, Widget, Metadata};

/// Contains all commands understood by the ui
#[derive(Debug, Serialize, Deserialize)]
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

struct UiClient {}
impl UiClient {
    #[tracing::instrument(skip_all)]
    pub async fn run_ui_client(self, mut rx: Receiver<Command>) {
        // Get new messages if available
        if let Some(cmd) = rx.recv().await {
            // Forward to websocket
            // TODO
        }
    }
}

pub fn run_ui_client(rx: Receiver<Command>) {
    let client = UiClient { };
    tokio::spawn(client.run_ui_client(rx));
}
