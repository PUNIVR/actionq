use tokio::sync::{mpsc, oneshot, broadcast};

use crate::pose::{PoseProxy, PoseEventSender, PoseEventSink};
use prepose::PoseData;

enum Command {

    /// Signal to the session controller that we are interested in receiving data
    ConnectToDataStream { 
        respond_to: oneshot::Sender<broadcast::Receiver<PoseData>>
    },

    SessionStart,
    SessionEnd,
    ExerciseStart { 
        exercise_id: usize,
    },
    ExerciseEnd
}

#[derive(PartialEq)]
enum SessionState {
    Idle, 
    SessionIdle,
    ExerciseRunning 
}

struct Session {
    receiver: mpsc::Receiver<Command>,
    data_sender: broadcast::Sender<PoseData>,
    state: SessionState,
    
    in_progress: bool,
    pose_receiver: mpsc::Receiver<PoseData>,
    pose: PoseProxy,
}

#[derive(Clone)]
pub struct SessionProxy(mpsc::Sender<Command>);
impl SessionProxy {

    pub async fn connect_output_stream(&self) -> broadcast::Receiver<PoseData> {
        let (tx, rx) = oneshot::channel();
        self.0.send(Command::ConnectToDataStream { respond_to: tx }).await.unwrap();
        rx.await.unwrap()
    }

    pub async fn session_start(&self) {
        self.0.send(Command::SessionStart).await.unwrap();
    }
    pub async fn session_end(&self) {
        self.0.send(Command::SessionEnd).await.unwrap();
    }
    pub async fn exercise_start(&self, exercise_id: usize) {
        self.0.send(Command::ExerciseStart {
            exercise_id
        }).await.unwrap();
    }
    pub async fn exercise_end(&self) {
        self.0.send(Command::ExerciseEnd).await.unwrap();
    }
}

impl Session {
    fn instantiate(pose: &PoseProxy, pose_receiver: mpsc::Receiver<PoseData>) -> (Self, SessionProxy) {

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
                in_progress: false,
                pose: pose.clone(),
            },
            SessionProxy(tx),
        )
    }

    async fn handle_command(&mut self, cmd: Command) {
        match cmd {

            Command::ConnectToDataStream { respond_to } => {
                let data_receiver = self.data_sender.subscribe();
                respond_to.send(data_receiver).unwrap();
            }

            Command::SessionStart => {
                if self.state != SessionState::Idle {
                    println!("Session: invalid state for session start");
                    return;
                }
                
                // TODO: load exercises collection

                self.state = SessionState::SessionIdle;
                println!("Session: session start");
            },
            Command::ExerciseStart { exercise_id } => {
                if self.state != SessionState::SessionIdle {
                    println!("Session: invalid state for exercise start");
                    return;
                }

                self.pose.inference_start().await;
                self.state = SessionState::ExerciseRunning;
                println!("Session: exercise start ({})", exercise_id);
            },
            Command::ExerciseEnd => {
                if self.state != SessionState::ExerciseRunning {
                    println!("Session: invalid state for exercise end");
                    return;
                }

                self.pose.inference_end().await;

                self.state = SessionState::SessionIdle;
                println!("Session: exercise end");
            }
            Command::SessionEnd => {
                if self.state != SessionState::SessionIdle {
                    println!("Session: invalid state for session end");
                    return;
                }

                // TODO: save to database

                println!("Session: session end");
                self.state = SessionState::Idle;
            },
            _ => todo!()
        }
    }

    async fn run(mut self) {
        println!("Session - run");
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
                    if let Some(pose) = pose_data {
                        // TODO: run motion analyzer
                        // Broadcast pose to connections
                        self.data_sender.send(pose).unwrap();
                    }
                }
            }
        }
    }
}

pub fn run_session(pose: &PoseProxy, pose_receiver: mpsc::Receiver<PoseData>) -> SessionProxy {
    let (session, proxy) = Session::instantiate(pose, pose_receiver);
    tokio::spawn(session.run());
    proxy
}
