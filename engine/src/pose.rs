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
    InferenceStart {
        outputs: Vec<mpsc::Sender<PoseData>>
    },
    InferenceStop,

    /// Request a direct connection to the pose output
    RequestDataStream {
        respond_to: oneshot::Sender<PoseEventSink>,
    },
}

#[derive(Clone)]
pub struct PoseProxy(mpsc::Sender<Command>);
impl PoseProxy {

    pub async fn inference_start(&self, outputs: Vec<mpsc::Sender<PoseData>>) {
        self.0.send(Command::InferenceStart {
            outputs
        }).await.unwrap();
    }

    pub async fn inference_end(&self) {
        self.0.send(Command::InferenceStop).await.unwrap();
    }

    pub async fn request_data_stream(&self) -> PoseEventSink {
        let (tx, rx) = oneshot::channel();
        self.0
            .send(Command::RequestDataStream { respond_to: tx })
            .await
            .unwrap();
        rx.await.unwrap()
    }
}

/// Pose estimator and analyzer
struct Pose {
    receiver: mpsc::Receiver<Command>,
    outputs: Option<Vec<mpsc::Sender<PoseData>>>,
    engine: PoseEstimator,
    is_running: bool,
}

impl Pose {
    pub fn instantiate() -> (Self, PoseProxy) {
        let (sender, receiver) = mpsc::channel(100);
        let engine = PoseEstimator::new(
            "network/pose_resnet18_body.onnx", 
            "network/human_pose.json", 
            "network/colors.txt"
        );
        (
            Pose {
                receiver,
                outputs: None,
                engine,
                is_running: false
            },
            PoseProxy(sender),
        )
    }

    fn handle_message(&mut self, msg: Command) {
        match msg {
            Command::InferenceStart { outputs } => {
                self.engine.inference_start("/dev/video0");
                self.is_running = true;
                self.outputs = Some(outputs);
            }
            Command::InferenceStop => {
                self.engine.inference_end();
                self.is_running = false;
                self.outputs = None;
            }
            /// Request a direct connection to the pose output
            Command::RequestDataStream { respond_to } => todo!()
        }
    }

    pub async fn run(mut self) {
        tokio::task::block_in_place(move || {
            loop {
                if self.is_running {
                    //println!("pose - inference");
                    
                    // Generate a pose estimation and output to channel
                    let pose = self.engine.inference_step();
                    if let Some(ref outputs) = self.outputs {
                        if let Some(pose) = pose {
                            
                            // We don't care if there aren't any receiver...
                            for sender in outputs {
                                let _ = sender.blocking_send(pose.clone());
                            }
                        }
                    }

                    // Try handle command
                    if let Ok(msg) = self.receiver.try_recv() {
                        self.handle_message(msg);
                    }

                    // How mutch to wait between frames
                    //std::thread::sleep(std::time::Duration::from_millis(10));

                } else {
                    if let Some(msg) = self.receiver.blocking_recv() {
                        println!("pose - received message");
                        self.handle_message(msg);
                    }
                }
            }        
        })
    }
}

pub fn run_human_pose_estimator() -> PoseProxy {
    let (engine, proxy) = Pose::instantiate();
    tokio::spawn(engine.run());
    proxy
}
