#![allow(dead_code, unused_imports)]

use glam::Vec2;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::ops::Deref;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::common::RequestExerciseReps;
use crate::firebase::model;
use crate::firebase::FirebaseProxy;
use crate::pose::{PoseEventSender, PoseEventSink, PoseProxy};
use crate::ui::UiProxy;

use actionq_common::*;
use actionq_motion::{LuaExercise, StateEvent, StateOutput, StateWarning};

pub enum Command {
    SessionStart {
        exercises: Vec<RequestExerciseReps>,
        save: bool,
    },
    SetPlayState {
        running: bool,
    },
    SessionEnd,
}

/*
/// Creates a Skeleton from FrameData and a joint mapping
pub fn framedata_to_skeleton(data: &CaptureData, joints: &[&str]) -> Skeleton {
    let mut result = HashMap::<String, Vec2>::new();
    for (i, joint) in joints.iter().enumerate() {

        // Skip all joint at position (0.0, 0.0) as they are not available
        let coords = data.keypoints[i];
        if coords.x != 0.0 && coords.y != 0.0 {
            result.insert(String::from(*joint), data.keypoints[i]);
        }
    }
    result
}
*/

/*
"nose" => 0
"left_eye" => 1
"right_eye" => 2
"left_ear" => 3
"right_ear" => 4
"left_shoulder" => 5
"right_shoulder" => 6
"left_elbow" => 7
"right_elbow" => 8
"left_wrist" => 9
"right_wrist" => 10
"left_hip" => 11
"right_hip" => 12
"left_knee" => 13
"right_knee" => 14
"left_ankle" => 15
"right_ankle" => 16
"neck" => 17
*/

const DOWN: Vec2 = Vec2::new(0.0, -1.0);
const UP: Vec2 = Vec2::new(0.0, 1.0);
const LEFT: Vec2 = Vec2::new(-1.0, 0.0);
const RIGHT: Vec2 = Vec2::new(1.0, 0.0);

#[derive(Debug)]
struct SessionState {
    /// All the exercises to execute during this session
    pub exercises: Vec<LuaExercise>,
    /// The currently active exercise index
    pub current_idx: usize,
    /// If true then run analyzer and store logs, otherwise skip frames analysis
    pub running: bool,
}

impl SessionState {
    /// Process a frame, returns the following:
    /// - exercise_is_complete, session_is_complete, StateOutput
    pub fn process(&mut self, capture: &CaptureData) -> (bool, bool, Option<StateOutput>) {
        let exercise = &mut self.exercises[self.current_idx];
        let (finished, output) = exercise
            .process(capture)
            .expect("Unable to process current frame");

        let mut completed = false;
        if finished {
            if self.current_idx < self.exercises.len() - 1 {
                self.current_idx += 1;
            } else {
                completed = true;
            }
        }

        (finished, completed, output)
    }

    /// Get current exercise name
    pub fn current_exercise_name(&self) -> String {
        self.exercises[self.current_idx].name.clone()
    }

    /// get current exercise
    pub fn current_repetitions(&self) -> (u32, u32) {
        let ex = &self.exercises[self.current_idx];
        (ex.repetitions_target, ex.repetitions)
    }
}

/// Convert a LuaExercise into a Firestore SessionExercise
impl From<&LuaExercise> for model::SessionExercise {
    fn from(other: &LuaExercise) -> Self {
        Self {
            Exercise: other.name.clone(),
            ExerciseTimestamp: String::new(),
            NumRepetitionsDone: other.repetitions,
            Poses: other
                .frames
                .iter()
                .enumerate()
                .map(|(i, (sk, _))| model::FramePose {
                    FrameId: i,
                    Keypoints: sk.iter().map(|(k, v)| (k.clone(), (v.x, v.y))).collect(),
                })
                .collect(),
        }
    }
}

/// Convert a SessionState to a Firestore Session
impl From<&SessionState> for model::Session {
    fn from(other: &SessionState) -> Self {
        Self {
            Timestamp: String::new(),
            Exercises: other
                .exercises
                .iter()
                .map(model::SessionExercise::from)
                .collect(),
        }
    }
}

#[derive(Debug)]
struct Session {
    /// Channel to receive session commands
    receiver: mpsc::Receiver<Command>,
    /// The current session, if present
    session: Option<SessionState>,
    /// If true then entirelly skip processing frames,
    /// this is necessary for resiliency during session end
    ignore_frames: bool,
    /// Channel used to receive poses from the HPE
    pose_receiver: mpsc::Receiver<CaptureData>,
    /// Proxy to command the HPE system
    pose: PoseProxy,
    /// Proxy to command the TV's ui.
    ui: UiProxy,
    /// Proxy to command the firebase database
    firebase: FirebaseProxy,
    // Broadcast the pose analysis, useful in future for more developed UIs
    //_data_sender: broadcast::Sender<SessionPoseData>,
}

#[derive(Clone, Debug)]
pub struct SessionProxy(mpsc::Sender<Command>);
impl SessionProxy {
    // TODO
    //pub async fn connect_output_stream(&self) -> broadcast::Receiver<SessionPoseData> {
    //    let (tx, rx) = oneshot::channel();
    //    self.0
    //        .send(Command::ConnectToDataStream { respond_to: tx })
    //        .await
    //        .unwrap();
    //    rx.await.unwrap()
    //}

    pub async fn session_start(&self, exercises: Vec<RequestExerciseReps>, save: bool) {
        self.0
            .send(Command::SessionStart { exercises, save })
            .await
            .unwrap();
    }

    pub async fn session_end(&self) {
        self.0.send(Command::SessionEnd).await.unwrap();
    }

    pub async fn set_play_state(&self, running: bool) {
        self.0
            .send(Command::SetPlayState { running })
            .await
            .unwrap();
    }
}

impl Session {
    fn instantiate(
        pose: &PoseProxy,
        pose_receiver: mpsc::Receiver<CaptureData>,
        ui: UiProxy,
        firebase: FirebaseProxy,
    ) -> (Self, SessionProxy) {
        // Broadcast channel used to send analyzed data
        //let (final_sender, final_receiver) = broadcast::channel(100);
        //drop(final_receiver);

        // Channel used to comunicate with actor
        let (tx, rx) = mpsc::channel(100);
        (
            Self {
                receiver: rx,
                //_data_sender: final_sender,
                pose_receiver,
                ignore_frames: true,
                session: None,
                pose: pose.clone(),
                ui,
                firebase,
            },
            SessionProxy(tx),
        )
    }

    #[tracing::instrument(err)]
    fn send_cec_signal(&self) -> std::io::Result<std::process::Output> {
        std::process::Command::new("./turn_on.sh").output()
    }

    #[tracing::instrument(skip_all, fields(exercises, save))]
    async fn session_start(&mut self, exercises: Vec<RequestExerciseReps>, save: bool) {
        // There is already a session active!
        if let Some(_) = &self.session {
            tracing::warn!("invalid state for session start");
            return;
        }

        // Send CEC command to turn on the TV
        self.send_cec_signal().expect("unable to turn on TV");

        // Load exercises collection
        let mut states: Vec<LuaExercise> = vec![];
        for e in &exercises {
            // Obtain the exercise descriptor from the database
            let descriptor = self.firebase.get_exercise(&e.exercise_id).await;
            if let Some(descriptor) = descriptor {
                tracing::info!("loaded descriptor for exercise {}", &e.exercise_id);
                states.push(
                    LuaExercise::from_string(
                        descriptor.fsm,
                        descriptor.name,
                        descriptor.description,
                        e.num_repetitions,
                    )
                    .expect("Unable to create LuaExercise"),
                );
            } else {
                tracing::error!("unable to find exercise with id: {}", e.exercise_id);
                return;
            }
        }

        self.session = Some(SessionState {
            current_idx: 0,
            running: true,
            exercises: states,
        });

        // Notify other actors to start HPE inference and visualization
        self.pose.inference_start().await;

        // Send notification to the UI, new session started, and first exercise
        let session = self.session.as_ref().unwrap();

        // Start new session on the ui
        self.ui
            .send(crate::ui::Command::SessionStart {
                exercises_count: session.exercises.len(),
                exercise_ids: session.exercises.iter().map(|e| e.name.clone()).collect(),
                resolution: (1280, 720), //TODO: Make dynamic
                frame_rate: 30,          //TODO: Make dynamic
            })
            .await;

        // Start first exercise
        self.ui
            .send(crate::ui::Command::ExerciseStart {
                exercise_id: session.current_exercise_name(),
                repetitions_target: session.exercises[0].repetitions_target,
            })
            .await;

        self.ignore_frames = false;
        tracing::info!("session started");
    }

    #[tracing::instrument(skip_all, fields(running))]
    fn set_play_state(&mut self, running: bool) {
        if let Some(session) = self.session.as_mut() {
            tracing::info!("set play state to {}", running);
            session.running = running;
        } else {
            tracing::warn!("invalid state for changing play state: no session active");
        }
    }

    #[tracing::instrument(skip_all)]
    async fn session_end(&mut self) {
        if let Some(session) = &self.session {
            // Notify other actors to stop
            self.pose.inference_end().await;
            self.ui.send(crate::ui::Command::SessionEnd).await;
            self.ignore_frames = true;

            // Save to database
            self.firebase
                .store_session(model::Session::from(session))
                .await;

            self.session = None;
            tracing::info!("session ended");
            return;
        }
    }

    #[tracing::instrument(skip_all, fields(cmd))]
    async fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::SessionStart { exercises, save } => self.session_start(exercises, save).await,
            Command::SetPlayState { running } => self.set_play_state(running),
            Command::SessionEnd => self.session_end().await,
        }
    }

    #[tracing::instrument(skip_all)]
    async fn run_session(mut self) {
        // TODO: use real deltatime

        self.ui
            .send(crate::ui::Command::ExerciseStart {
                exercise_id: (),
                repetitions_target: (),
            })
            .await;

        const DELTATIME: f32 = 0.2;
        loop {
            tokio::select! {

                // Handle commands from other actors
                cmd_data = self.receiver.recv() => {
                    if let Some(cmd) = cmd_data {
                        self.handle_command(cmd).await;
                    }
                },

                // Handle data from pose estimator
                data = self.pose_receiver.recv() => {
                    if self.ignore_frames {
                        continue;
                    }

                    let capture = data.unwrap();

                    // If the pose estimator is running then we must have a current session!
                    let session = self.session.as_mut().expect("");
                    if session.running {
                        tracing::trace!("running exercise analyzer");

                        let (finished, completed, output) = session.process(&capture);
                        let (repetitions_target, repetitions) = session.current_repetitions();

                        //println!("{:?}", output);

                        // Send progress to UI
                        self.ui
                            .send(crate::ui::Command::ExerciseUpdate {
                                metadata: (),
                                skeleton: capture,
                                repetitions: (),
                                frame: (),
                            })
                            .await;

                        match (finished, completed) {
                            // Close session
                            (true, true) => {
                                tracing::info!("session completed");
                                self.session_end().await;

                                // Save session to database
                                //self.firebase.store_session(
                                //    model::Session::from(session));
                            },
                            // Next exercise
                            (true, false) => {
                                tracing::info!("moving to next exercise");
                                self.pose.inference_end().await;
                                self.ui.send(crate::ui::Command::ExerciseEnd).await;

                                self.pose.inference_start().await;
                                self.ui.send(crate::ui::Command::ExerciseStart { exercise_id: (), repetitions_target: () }).await;
                            },
                            _ => {}
                        }

                    }
                }
            }
        }
    }
}

pub fn run_session(
    pose: &PoseProxy,
    pose_receiver: mpsc::Receiver<CaptureData>,
    ui: UiProxy,
    firebase: FirebaseProxy,
) -> SessionProxy {
    let (session, proxy) = Session::instantiate(pose, pose_receiver, ui, firebase);
    tokio::spawn(session.run_session());
    proxy
}
