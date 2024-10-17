use tokio::sync::{mpsc, oneshot, broadcast};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use glam::Vec2;
use std::ops::Deref;

use crate::pose::{PoseEventSender, PoseEventSink, PoseProxy};
use crate::ui::UiProxy;
use videopose::{FrameData, Framebuffer};
use motion::{
    MotionAnalyzer, 
    Exercise, 
    Transition, 
    Warning, 
    StateId, 
    GenControlFactors, 
    ControlFactorMap, 
    MappedCondition, 
    Condition,
    Event
};

enum Command {
    /// Signal to the session controller that we are interested in receiving data
    ConnectToDataStream { 
        respond_to: oneshot::Sender<broadcast::Receiver<SessionPoseData>>
    },

    SessionStart,
    SessionEnd,
    ExerciseStart {
        exercise_id: usize,
    },
    ExerciseEnd,
}

#[derive(PartialEq, Debug)]
enum SessionState {
    Idle,
    SessionIdle,
    ExerciseRunning,
}

#[derive(Debug, Clone, Deserialize)]
struct JsonState {
    /// Unique name
    name: String,
    /// All transitions to other states
    transitions: Vec<motion::Transition>,
    /// Warnings active only in this state
    warnings: Vec<motion::Warning>,
}

/// An exercise specified using json
#[derive(Debug, Clone, Deserialize)]
struct JsonExercise {
    /// All the possible states
    states: Vec<JsonState>,
    /// Initial state the analyzer will start in
    initial_state: String,
    /// Global warnings active in all states
    warnings: Vec<motion::Warning>,
}

impl JsonExercise {
    
    /// A very simple example exercise 
    pub fn simple() -> Self {
        Self {
            states: vec![
                JsonState {
                    name: "start".into(),
                    warnings: vec![],
                    transitions: vec![
                        Transition {
                            to: "down".into(),
                            emit: vec![],
                            conditions: vec![
                                motion::MappedCondition {
                                    control_factor: "arm_angle_l".into(),
                                    condition: motion::Condition::InRange {
                                        range: (0.0..30.0),
                                    },
                                },
                                motion::MappedCondition {
                                    control_factor: "arm_angle_r".into(),
                                    condition: motion::Condition::InRange {
                                        range: (0.0..30.0),
                                    },
                                },
                            ],
                        },
                    ],
                },
                JsonState {
                    name: "up".into(),
                    warnings: vec![],
                    transitions: vec![
                        Transition {
                            to: "down".into(),
                            emit: vec![motion::Event::RepetitionComplete],
                            conditions: vec![
                                motion::MappedCondition {
                                    control_factor: "arm_angle_l".into(),
                                    condition: motion::Condition::InRange {
                                        range: (0.0..30.0),
                                    },
                                },
                                motion::MappedCondition {
                                    control_factor: "arm_angle_r".into(),
                                    condition: motion::Condition::InRange {
                                        range: (0.0..30.0),
                                    },
                                },
                            ],
                        },
                    ],
                },
                JsonState {
                    name: "down".into(),
                    warnings: vec![],
                    transitions: vec![
                        motion::Transition {
                            to: "up".into(),
                            emit: vec![],
                            conditions: vec![
                                motion::MappedCondition {
                                    control_factor: "arm_angle_l".into(),
                                    condition: motion::Condition::InRange {
                                        range: (45.0..90.0),
                                    },
                                },
                                motion::MappedCondition {
                                    control_factor: "arm_angle_r".into(),
                                    condition: motion::Condition::InRange {
                                        range: (45.0..90.0),
                                    },
                                },
                            ],
                        },
                    ],
                },
            ],
            initial_state: "start".into(),
            warnings: vec![]
        }
    }

    pub fn from_file<S: AsRef<str>>(filepath: S) -> Self {
        let string = fs::read_to_string(filepath.as_ref()).unwrap();
        Self::from_str(string)
    }

    pub fn from_str<S: AsRef<str>>(string: S) -> Self {
        serde_json::from_str::<Self>(string.as_ref()).unwrap()
    }
}

impl Exercise for JsonExercise {

    fn states(&self) -> Vec<String> {
        self.states.iter()
            .map(|s| s.name.clone())
            .collect()
    }

    fn initial_state(&self) -> String {
        self.initial_state.clone()
    }

    fn state_transitions(&self, state: &String) -> Vec<Transition> {
        self.states.iter()
            .filter(|s| s.name == *state)
            .flat_map(|s| s.transitions.clone())
            .collect()
    }

    fn state_warnings(&self, state: &StateId) -> Vec<Warning> {
        self.states.iter()
            .filter(|s| s.name == *state)
            .flat_map(|s| s.warnings.clone())
            .collect()
    }

    fn global_warnings(&self) -> Vec<Warning> {
        self.warnings.clone()
    }
}

#[derive(Debug, Clone)]
pub struct SessionPoseData(FrameData);
impl Deref for SessionPoseData {
    type Target = FrameData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Extract control factors from the pose data
impl GenControlFactors for SessionPoseData {
    fn control_factors(&self) -> ControlFactorMap {

        // vector in the down direction
        let down = Vec2::new(0.0, -1.0);

        let ls = self.keypoint_from_name("left_shoulder").unwrap();
        let le = self.keypoint_from_name("left_elbow").unwrap();

        let la = (le - ls).normalize(); // shoulder to elbow
        let adl = la.dot(down).abs().acos().to_degrees();

        let rs = self.keypoint_from_name("right_shoulder").unwrap();
        let re = self.keypoint_from_name("right_elbow").unwrap();

        let ra = (re - rs).normalize(); // shoulder to elbow
        let adr = ra.dot(down).abs().acos().to_degrees();

        BTreeMap::from([
            ("arm_angle_l".into(), adl),
            ("arm_angle_r".into(), adr),
        ])
    }
}

#[derive(Debug)]
struct Session {
    receiver: mpsc::Receiver<Command>,
    data_sender: broadcast::Sender<SessionPoseData>,
    state: SessionState,

    in_progress: bool,
    pose_receiver: mpsc::Receiver<FrameData>,
    pose: PoseProxy,
    ui: UiProxy,

    /// Store the current exercise data
    current_exercise_data: Vec<SessionPoseData>,
    analyzer: Option<MotionAnalyzer<JsonExercise>>
}

#[derive(Clone, Debug)]
pub struct SessionProxy(mpsc::Sender<Command>);
impl SessionProxy {

    pub async fn connect_output_stream(&self) -> broadcast::Receiver<SessionPoseData> {
        let (tx, rx) = oneshot::channel();
        self.0
            .send(Command::ConnectToDataStream { respond_to: tx })
            .await
            .unwrap();
        rx.await.unwrap()
    }

    pub async fn session_start(&self) {
        self.0.send(Command::SessionStart).await.unwrap();
    }
    pub async fn session_end(&self) {
        self.0.send(Command::SessionEnd).await.unwrap();
    }
    pub async fn exercise_start(&self, exercise_id: usize) {
        self.0
            .send(Command::ExerciseStart { exercise_id })
            .await
            .unwrap();
    }
    pub async fn exercise_end(&self) {
        self.0.send(Command::ExerciseEnd).await.unwrap();
    }
}

impl Session {
    fn instantiate(
        pose: &PoseProxy,
        pose_receiver: mpsc::Receiver<FrameData>,
        ui: UiProxy
    ) -> (Self, SessionProxy) {

        // Broadcast channel used to send analyzed data
        let (final_sender, final_receiver) = broadcast::channel(100);
        drop(final_receiver);

        // Channel used to comunicate with actor
        let (tx, rx) = mpsc::channel(100);
        (
            Self {
                receiver: rx,
                data_sender: final_sender,
                pose_receiver,
                state: SessionState::Idle,
                current_exercise_data: vec![],
                analyzer: None,
                in_progress: false,
                pose: pose.clone(),
                ui: ui
            },
            SessionProxy(tx),
        )
    }

    #[tracing::instrument(skip_all, fields(cmd))]
    async fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::ConnectToDataStream { respond_to } => {
                let data_receiver = self.data_sender.subscribe();
                respond_to.send(data_receiver).unwrap();
            }

            Command::SessionStart => {
                if self.state != SessionState::Idle {
                    tracing::warn!("invalid state for session start");
                    return;
                }

                // TODO: load exercises collection

                self.state = SessionState::SessionIdle;
                tracing::info!("session started");
            }
            Command::ExerciseStart { exercise_id } => {
                if self.state != SessionState::SessionIdle {
                    tracing::warn!("invalid state for exercise start");
                    return;
                }

                // Setup motion analyzer and storage
                self.current_exercise_data.clear();
                self.analyzer = Some(MotionAnalyzer::new(JsonExercise::simple()));
                self.pose.inference_start().await;

                // Start visualization
                self.ui.exercise_show(exercise_id).await;

                self.state = SessionState::ExerciseRunning;
                tracing::info!("exercise started ({})", exercise_id);
            }
            Command::ExerciseEnd => {
                if self.state != SessionState::ExerciseRunning {
                    tracing::warn!("invalid state for exercise end");
                    return;
                }

                // Stop pose estimator
                self.pose.inference_end().await;

                // Stop visualization
                self.ui.exercise_stop().await;

                self.state = SessionState::SessionIdle;
                tracing::info!("exercise ended");
            }
            Command::SessionEnd => {
                if self.state != SessionState::SessionIdle {
                    tracing::warn!("invalid state for session end");
                    return;
                }

                // TODO: save to database

                self.state = SessionState::Idle;
                tracing::info!("session ended");
            }
            _ => todo!(),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn run_session(mut self) {
        loop {
            tokio::select! {

                // Handle commands from other actors
                cmd_data = self.receiver.recv() => {
                    if let Some(cmd) = cmd_data {
                        self.handle_command(cmd).await;
                    }
                },

                // Handle data from pose estimator
                pose_data = self.pose_receiver.recv() => {
                    if let Some(pose_prepose) = pose_data {
                        tracing::trace!("receive pose data and framebuffer");

                        // Send framebuffer to ui first
                        // FIXME: remove unnecessary clone, after this we only need the skeleton, not the framebuffer
                        let framebuffer = &pose_prepose.framebuffer;
                        self.ui.display_frame(framebuffer.storage.clone(), framebuffer.size.0 as usize, framebuffer.size.1 as usize).await;
                        let _ = framebuffer;
                        
                        // Analyze only if there is a subject
                        let pose = SessionPoseData(pose_prepose);
                        if pose.subjects != 0 {

                            // Save pose for later storage
                            self.current_exercise_data.push(pose.clone());

                            // TODO: use real deltatime
                            let deltatime = 0.1;
                            if let Some(analyzer) = &mut self.analyzer {
                                // println!("{:?}", pose.control_factors());
                                let progress = analyzer.progress(deltatime, &pose);
                                println!("{:?}", progress);
                            } else {
                                println!("warning: running exercise without an analyzer!")
                            }

                            // Broadcast pose to connections
                            // TODO: add analyzer output to broadcast
                            self.data_sender.send(pose).unwrap();

                        } else {
                            println!("not subject in frame!");
                        }
                    }
                }
            }
        }
    }
}

pub fn run_session(pose: &PoseProxy, pose_receiver: mpsc::Receiver<FrameData>, ui: UiProxy) -> SessionProxy {
    let (session, proxy) = Session::instantiate(pose, pose_receiver, ui);
    tokio::spawn(session.run_session());
    proxy
}
