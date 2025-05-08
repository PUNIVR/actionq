use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use actionq_common::CaptureData;
use actionq_zed::*;

// Receiver for HPE data
#[derive(Debug)]
pub struct PoseEventSink(pub broadcast::Receiver<CaptureData>);
// Send HPE data
#[derive(Debug)]
pub struct PoseEventSender(pub broadcast::Sender<CaptureData>);

pub enum Command {
    InferenceStart,
    InferenceStop,
}

#[derive(Clone, Debug)]
pub struct PoseProxy {
    commands: mpsc::Sender<Command>,
}

impl PoseProxy {
    pub async fn inference_start(&self) {
        self.commands.send(Command::InferenceStart).await.unwrap();
    }

    pub async fn inference_end(&self) {
        self.commands.send(Command::InferenceStop).await.unwrap();
    }
}

/// Pose estimator and analyzer
#[derive(Debug)]
struct Pose {
    cmd_receiver: mpsc::Receiver<Command>,
    data_sender: mpsc::Sender<CaptureData>,
    is_running: bool,
}

impl Pose {

    #[tracing::instrument]
    pub fn instantiate() -> (Self, PoseProxy, mpsc::Receiver<CaptureData>) {
        
        // Channel for commands
        let (cmd_sender, cmd_receiver) = mpsc::channel(100);
        // Channel for data output
        let (data_sender, data_receiver) = mpsc::channel(100);

        actionq_zed::initialize();
        println!("ZED ready");

        (
            Pose {
                cmd_receiver,
                data_sender,
                is_running: false,
            },
            PoseProxy {
                commands: cmd_sender,
            },
            data_receiver,
        )
    }

    #[tracing::instrument(skip_all, fields(msg))]
    fn handle_message(&mut self, msg: Command) {
        match msg {
            Command::InferenceStart => {
                if !self.is_running {
                    tracing::info!("inference started");
                    self.is_running = true;
                }
            }
            Command::InferenceStop => {
                if self.is_running {
                    tracing::info!("inference ended");
                    self.is_running = false;
                }
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn run_pose_estimator(mut self) {
        tokio::task::block_in_place(move || {
            loop {
                if self.is_running {

                    // Generate a pose estimation and output to channel
                    let frame_data = actionq_zed::extract();
                    self.data_sender.blocking_send(frame_data).unwrap();

                    // Try handle command
                    if let Ok(msg) = self.cmd_receiver.try_recv() {
                        self.handle_message(msg);
                    }

                } else {
                    if let Some(msg) = self.cmd_receiver.blocking_recv() {
                        tracing::trace!("received message");
                        self.handle_message(msg);
                    }
                }
            }
        })
    }
}

pub fn run_human_pose_estimator() -> (PoseProxy, mpsc::Receiver<CaptureData>) {
    let (engine, proxy, pose_receiver) = Pose::instantiate();
    tokio::spawn(engine.run_pose_estimator());
    (proxy, pose_receiver)
}
