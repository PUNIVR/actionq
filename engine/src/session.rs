use tokio::sync::{mpsc, oneshot, broadcast};

use crate::pose::{PoseProxy, PoseEventSender, PoseEventSink};
use prepose::PoseData;

enum Command {

    /// Signal to the session controller that we are interested in receiving data
    ConnectToDataStream { 
        respond_to: oneshot::Sender<mpsc::Receiver<PoseData>>
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
    output_channels: Vec<mpsc::Sender<PoseData>>,
    state: SessionState,
    in_progress: bool,

    pose_proxy: PoseProxy,
}

#[derive(Clone)]
pub struct SessionProxy(mpsc::Sender<Command>);
impl SessionProxy {
    pub async fn connect_pose_stream(&self) -> mpsc::Receiver<PoseData> {
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
    fn instantiate(pose_proxy: &PoseProxy) -> (Self, SessionProxy) {
        // Channel used to comunicate with actor
        let (tx, rx) = mpsc::channel(100);
        (
            Self {
                receiver: rx,
                output_channels: vec![],
                state: SessionState::Idle,
                in_progress: false,
                pose_proxy: pose_proxy.clone(),
            },
            SessionProxy(tx),
        )
    }

    async fn handle_command(&mut self, cmd: Command) {
        match cmd {

            Command::ConnectToDataStream { respond_to } => {
                let (tx, rx) = mpsc::channel(100);
                self.output_channels.push(tx);
                respond_to.send(rx).unwrap();
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

                // TODO: handle multiple connections to inference data
                self.pose_proxy.inference_start(self.output_channels.clone()).await;

                self.state = SessionState::ExerciseRunning;
                println!("Session: exercise start ({})", exercise_id);
            },
            Command::ExerciseEnd => {
                if self.state != SessionState::ExerciseRunning {
                    println!("Session: invalid state for exercise end");
                    return;
                }

                self.pose_proxy.inference_end().await;

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
        while let Some(cmd) = self.receiver.recv().await {
            self.handle_command(cmd).await;
        }
    }
}

pub fn run_session(pose_proxy: &PoseProxy) -> SessionProxy {
    let (session, proxy) = Session::instantiate(pose_proxy);
    tokio::spawn(session.run());
    proxy
}
