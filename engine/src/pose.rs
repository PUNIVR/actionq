use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use prepose::*;

// Receiver for HPE data
#[derive(Debug)]
pub struct PoseEventSink(pub broadcast::Receiver<PoseData>);
// Send HPE data
#[derive(Debug)]
pub struct PoseEventSender(pub broadcast::Sender<PoseData>);

pub enum Command {
    InferenceStart,
    InferenceStop,
}

#[derive(Clone)]
pub struct PoseProxy {
    commands: mpsc::Sender<Command>
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
struct Pose {
    cmd_receiver: mpsc::Receiver<Command>,
    data_sender: mpsc::Sender<PoseData>,
    engine: PoseEstimator,
    is_running: bool,
}

impl Pose {
    pub fn instantiate() -> (Self, PoseProxy, mpsc::Receiver<PoseData>) {

        // Channel for commands
        let (cmd_sender, cmd_receiver) = mpsc::channel(100);

        // Channel for data output
        let (data_sender, data_receiver) = mpsc::channel(100);

        // Connect to prepose library
        let engine = PoseEstimator::new(
            "network/pose_resnet18_body.onnx", 
            "network/human_pose.json", 
            "network/colors.txt"
        );
        
        (
            Pose {
                cmd_receiver,
                data_sender,
                engine,
                is_running: false
            },
            PoseProxy {
                commands: cmd_sender,
            },
            data_receiver
        )
    }

    fn handle_message(&mut self, msg: Command) {
        match msg {
            Command::InferenceStart => {
                if !self.is_running {
                    self.engine.inference_start("/dev/video0");
                    self.is_running = true;
                }
            }
            Command::InferenceStop => {
                if self.is_running {
                    self.engine.inference_end();
                    self.is_running = false;
                }
            }
        }
    }

    pub async fn run(mut self) {
        tokio::task::block_in_place(move || {
            loop {
                if self.is_running {
                    
                    // Generate a pose estimation and output to channel
                    let pose = self.engine.inference_step();
                    if let Some(pose) = pose {                            
                        self.data_sender.blocking_send(pose).unwrap();
                    }

                    // Try handle command
                    if let Ok(msg) = self.cmd_receiver.try_recv() {
                        self.handle_message(msg);
                    }

                    // How mutch to wait between frames
                    //std::thread::sleep(std::time::Duration::from_millis(10));

                } else {
                    if let Some(msg) = self.cmd_receiver.blocking_recv() {
                        println!("pose - received message");
                        self.handle_message(msg);
                    }
                }
            }        
        })
    }
}

pub fn run_human_pose_estimator() -> (PoseProxy, mpsc::Receiver<PoseData>) {
    let (engine, proxy, pose_receiver) = Pose::instantiate();
    tokio::spawn(engine.run());
    (proxy, pose_receiver)
}
