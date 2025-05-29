#![allow(dead_code)]

use eframe::{egui, App, NativeOptions};
use egui::{
    Align2, Button, Color32, FontFamily, FontId, Pos2, Rangef, Rect, Stroke, TextureOptions, Ui,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{Receiver, Sender};
use webp_animation::prelude::*;

use actionq_common::{CaptureData, Skeleton2D};
use actionq_motion::{LuaExercise, StateEvent, StateOutput, StateWarning, Widget};

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

impl App for MyUi {
    #[tracing::instrument(skip_all, fields(cmd))]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Get new messages if available
        if let Ok(cmd) = self.cmds.try_recv() {
            // Forward to websocket
        }
    }
}

pub fn run_ui_blocking(rx: Receiver<Command>) {
    eframe::run_native(
        "ActionQ",
        eframe_options(),
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            // static background image for now
            let image = egui::ColorImage::new([1280, 720], Color32::from_gray(0));

            Ok(Box::new(MyUi {
                is_running: false,
                repetition_count: 0,
                cmds: rx,
                exercise_gif: None,
                current_frame: Some(image),
                widgets: vec![],
                keypoints: vec![],
                help_text: None,
            }))
        }),
    )
    .expect("Unable to run eframe");
}
