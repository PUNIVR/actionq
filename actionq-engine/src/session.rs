use tokio::sync::{mpsc, oneshot, broadcast};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use glam::Vec2;
use std::ops::Deref;

use crate::pose::{PoseEventSender, PoseEventSink, PoseProxy};
use crate::common::RequestExerciseReps;
use crate::firebase::FirebaseProxy;
use crate::exercise::JsonExercise;
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
    ProgresState,
    Event
};

pub enum Command {
    SessionStart {
        exercises: Vec<RequestExerciseReps>,
        save: bool
    },
    SetPlayState {
        running: bool,
    },
    SessionEnd,
}

#[derive(Debug, Clone)]
pub struct SessionPoseData(FrameData);
impl Deref for SessionPoseData {
    type Target = FrameData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

const DOWN:  Vec2 = Vec2::new( 0.0, -1.0);
const UP:    Vec2 = Vec2::new( 0.0,  1.0);
const LEFT:  Vec2 = Vec2::new(-1.0,  0.0);
const RIGHT: Vec2 = Vec2::new( 1.0,  0.0);

/// Degrees between two vectors
fn degrees(a: Vec2, b: Vec2) -> f32 {
    a.angle_to(b).to_degrees().abs()
}

impl SessionPoseData {

    fn arm_angle_with_body(&self, side: &str) -> f32 {

        let shoulder = self.keypoint_from_name(&format!("{}_shoulder", side)).unwrap();
        let elbow = self.keypoint_from_name(&format!("{}_elbow", side)).unwrap();

        // Direction from shoulder to elbow
        let dir = (elbow - shoulder).normalize(); 
        return dir.dot(DOWN).abs().acos().to_degrees()
    }

    fn inner_angle(&self, beg: &str, mid: &str, end: &str) -> f32 {

        let beg = self.keypoint_from_name(beg).unwrap();
        let mid = self.keypoint_from_name(mid).unwrap();
        let end = self.keypoint_from_name(end).unwrap();

        let mid2beg = (beg - mid).normalize();
        let mid2end = (end - mid).normalize();

        let angle = degrees(mid2beg, mid2end);
        return angle;
    }

    fn arm_inner_angle_left(&self) -> f32 {

        let shoulder = self.keypoint_from_name("left_shoulder").unwrap();
        let elbow = self.keypoint_from_name("left_elbow").unwrap();
        let wrist = self.keypoint_from_name("left_wrist").unwrap();

        let elbow2shoulder = (shoulder - elbow).normalize();
        let elbow2wrist = (wrist - elbow).normalize();

        degrees(elbow2shoulder, elbow2wrist)
    }

    fn arm_inner_angle_right(&self) -> f32 {

        let shoulder = self.keypoint_from_name("right_shoulder").unwrap();
        let elbow = self.keypoint_from_name("right_elbow").unwrap();
        let wrist = self.keypoint_from_name("right_wrist").unwrap();

        dbg!(shoulder, elbow, wrist);

        let elbow2shoulder = (shoulder - elbow).normalize();
        let wrist2elbow = (wrist - elbow).normalize();

        degrees(elbow2shoulder, wrist2elbow)
    }

    //fn arm_inner_angle(&self, side: &str) -> f32 {
    //    self.inner_angle(
    //        &format!("{}_shoulder", side),
    //        &format!("{}_elbow", side),
    //        &format!("{}_wrist", side)
    //    )
    //}

    /*
    /// Angle of the arm from the shoulder axis 
    fn arm_shoulder_angle(&self, side: &str) -> f32 {

        let neck = self.keypoint_from_name("neck").unwrap();
        let shoulder = self.keypoint_from_name(&format!("{}_shoulder", side)).unwrap();
        let elbow = self.keypoint_from_name(&format!("{}_elbow", side)).unwrap();

        let neck2shoulder = (shoulder - neck).normalize();
        let shoulder2elbow = (elbow - shoulder).normalize();

        return degrees(neck2shoulder, shoulder2elbow);
    }

    fn torso_down_vector(&self) -> Vec2 {

        let neck = self.keypoint_from_name("neck").unwrap();
        let mut mid_hip = self.keypoint_from_name("right_hip").unwrap() + self.keypoint_from_name("left_hip").unwrap();
        mid_hip /= 2.0; 

        return (mid_hip - neck).normalize();
    }

    /// Angle of the arm from the vertical body axis
    fn arm_torso_angle(&self, side: &str) -> f32 {

        let shoulder = self.keypoint_from_name(&format!("{}_shoulder", side)).unwrap();
        let elbow = self.keypoint_from_name(&format!("{}_elbow", side)).unwrap();
        let shoulder2elbow = (elbow - shoulder).normalize();
        
        let torso = self.torso_down_vector();
        return degrees(shoulder2elbow, torso);
    }
    */
}

/// Extract control factors from the pose data
impl GenControlFactors for SessionPoseData {

    //fn control_factors(&self) -> ControlFactorMap {
    //
    //    // vector in the down direction
    //    let down = Vec2::new(0.0, -1.0);
    //
    //    let ls = self.keypoint_from_name("left_shoulder").unwrap();
    //    let le = self.keypoint_from_name("left_elbow").unwrap();
    //
    //    let la = (le - ls).normalize(); // shoulder to elbow
    //    let adl = la.dot(down).abs().acos().to_degrees();
    //
    //    let rs = self.keypoint_from_name("right_shoulder").unwrap();
    //    let re = self.keypoint_from_name("right_elbow").unwrap();
    //
    //    let ra = (re - rs).normalize(); // shoulder to elbow
    //    let adr = ra.dot(down).abs().acos().to_degrees();
    //
    //    BTreeMap::from([
    //        ("arm_angle_l".into(), adl),
    //        ("arm_angle_r".into(), adr),
    //    ])
    //}

    fn control_factors(&self) -> ControlFactorMap {
        let result = BTreeMap::from([

            ("arm_angle_l".into(), self.arm_angle_with_body("left")),
            ("arm_angle_r".into(), self.arm_angle_with_body("right")),

            // The inner angle of the arms
            ("arm_inner_angle_l".into(), self.arm_inner_angle_left()),
            //("arm_inner_angle_r".into(), self.arm_inner_angle("right")),

            // Degrees from the horizontal shoulder axis and the arm axis
            //("arm_horiz_angle_l".into(), 180.0 - self.arm_shoulder_angle("left")),
            //("arm_horiz_angle_r".into(), self.arm_shoulder_angle("right")),

            // Degrees from the vertical torso axis and the arm axis
            //("arm_vert_angle_l".into(), self.arm_torso_angle("left")),
            //("arm_vert_angle_r".into(), self.arm_torso_angle("right")),
        ]);

        // OK FUNZIONA! MANO DESTRA!
        let inner_L_angle = self.arm_inner_angle_left();
        dbg!(inner_L_angle);

        //dbg!(&self.keypoints);
        //let inner_R_angle = self.arm_inner_angle_right();
        //dbg!(inner_R_angle);

        //tracing::trace!("control_factors: {:?}", result);
        return result;
    }

}

#[derive(Debug)]
struct ExerciseState {
    /// Id of the exercise inside the database
    pub exercise_id: String,
    /// Analyzer to evaluate the exercise's execution
    pub analyzer: MotionAnalyzer<JsonExercise>,
    /// Number of repetitions done until now
    pub num_repetitions_done: u32,
    /// Number of repetitions to do
    pub num_repetitions: u32,
}

impl ExerciseState {
    pub fn completed(&self) -> bool {
        self.num_repetitions_done >= self.num_repetitions
    }

    /// Returns true if it is complete
    pub fn add_repetition(&mut self) -> bool {
        self.num_repetitions_done += 1;
        self.completed()
    }
}

#[derive(Debug)]
struct SessionState {
    /// All the exercises to execute during this session
    pub exercises: Vec<ExerciseState>,
    /// The currently active exercise index
    pub current_idx: usize,
    /// If true then run analyzer and store logs, otherwise skip frames analysis
    pub running: bool,
}

impl SessionState {

    pub fn current_exercise_const(&self) -> &ExerciseState {
        &self.exercises[self.current_idx]
    }

    pub fn current_exercise(&mut self) -> &mut ExerciseState {
        &mut self.exercises[self.current_idx]
    }

    // Return true if there is a next exercise, in that case increment inner counter
    pub fn next_exercise(&mut self) -> bool {
        self.current_idx += 1;
        if self.current_idx < self.exercises.len() {
            true
        } else {
            false
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
    pose_receiver: mpsc::Receiver<FrameData>,
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
        self.0.send(Command::SessionStart { exercises, save }).await.unwrap();
    }

    pub async fn session_end(&self) {
        self.0.send(Command::SessionEnd).await.unwrap();
    }

    pub async fn set_play_state(&self, running: bool) {
        self.0.send(Command::SetPlayState { running }).await.unwrap();
    }
}

impl Session {
    fn instantiate(
        pose: &PoseProxy,
        pose_receiver: mpsc::Receiver<FrameData>,
        ui: UiProxy,
        firebase: FirebaseProxy
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
                firebase
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
        self.send_cec_signal()
            .expect("unable to turn on TV");

        // Load exercises collection
        let mut states: Vec<ExerciseState> = vec![];
        for e in &exercises {

            // Obtain the exercise descriptor from the database
            let descriptor = self.firebase.get_exercise(&e.exercise_id).await;
            if let Some(descriptor) = descriptor {
                tracing::info!("loaded descriptor for exercise {}", &e.exercise_id);
                let analyzer = MotionAnalyzer::new(JsonExercise::from_str(descriptor.fsm));
                states.push(ExerciseState {
                    exercise_id: e.exercise_id.clone(),
                    analyzer: analyzer,
                    num_repetitions: e.num_repetitions,
                    num_repetitions_done: 0
                });
            } else {
                tracing::error!("unable to find exercise with id: {}", e.exercise_id);
                return;
            }
        }

        self.session = Some(
            SessionState {
                current_idx: 0,
                running: true,
                exercises: states
            }
        );

        // Notify other actors to start HPE inference and visualization 
        self.pose.inference_start().await;
        let session = self.session.as_ref().unwrap();
        self.ui.exercise_show(session.current_exercise_const().exercise_id.clone()).await;
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
        // There is not session to end!
        if let None = self.session {
            tracing::warn!("invalid state for session end");
            return;
        }

        // Notify other actors to stop
        self.pose.inference_end().await;
        self.ui.exercise_stop().await;
        self.ignore_frames = true;

        // TODO: save to database
        // for now simulate a drop of the current session
        self.session = None;
        tracing::info!("session ended");
    }

    #[tracing::instrument(skip_all, fields(cmd))]
    async fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::SessionStart { exercises, save } => self.session_start(exercises, save).await,
            Command::SetPlayState { running } => self.set_play_state(running),
            Command::SessionEnd => self.session_end().await,
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
                    if self.ignore_frames {
                        continue;
                    }

                    // TODO: use real deltatime
                    const DELTATIME: f32 = 0.2;

                    // Analyze only if there is a subject
                    if let Some(pose_prepose) = pose_data {
                        let pose = SessionPoseData(pose_prepose);
                        if pose.subjects != 0 {

                            // If the pose estimator is running then we must have a current session!
                            let mut progress = None;
                            if let Some(session) = self.session.as_mut() {
                                // Analyze the movement
                                if session.running {
                                    tracing::trace!("running exercise analyzer");
                                    progress = Some(session.current_exercise().analyzer.progress(DELTATIME, &pose));
                                    tracing::trace!("{:?}", progress);
                                }
                            } else {
                                tracing::error!("The pose estimator is running without an active session!");
                                assert!(false); // This can not be
                            }

                            // Handle events
                            if let Some(progress) = progress {
                                for event in &progress.events {
                                    match event {
                                        Event::RepetitionComplete => {

                                            // Update repetition count and check if session is completed
                                            let session = self.session.as_mut().unwrap();
                                            if session.current_exercise().add_repetition() {
                                                tracing::info!("exercise completed");

                                                // Change exercise and check if session has ended
                                                if !session.next_exercise() {
                                                    tracing::info!("session completed");
                                                    self.session_end().await;
                                                } else {
                                                    tracing::info!("moving to next exercise");

                                                    self.pose.inference_end().await;
                                                    self.ui.exercise_stop().await;

                                                    self.pose.inference_start().await;
                                                    self.ui.exercise_show(session.current_exercise_const().exercise_id.clone()).await;
                                                }
                                            }

                                        }
                                    }
                                }

                                // Send progress to UI
                                self.ui.update(progress, pose.framebuffer.clone()).await;
                            }

                            // TODO: Broadcast pose to connections
                            // TODO: add analyzer output to broadcast
                            //self.data_sender.send(pose).unwrap();

                        } else {
                            println!("not subject in frame!");
                        }
                    }
                }
            }
        }
    }
}

pub fn run_session(pose: &PoseProxy, pose_receiver: mpsc::Receiver<FrameData>, ui: UiProxy, firebase: FirebaseProxy) -> SessionProxy {
    let (session, proxy) = Session::instantiate(pose, pose_receiver, ui, firebase);
    tokio::spawn(session.run_session());
    proxy
}
