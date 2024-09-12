use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use prepose::*;

pub struct PoseData {}

// Receiver for HPE data
pub struct PoseEventSink(broadcast::Receiver<PoseData>);

pub enum Command {
    InferenceStart,
    InferenceStop,

    /// Request a direct connection to the pose output
    RequestDataStream {
        respond_to: oneshot::Sender<PoseEventSink>,
    },
}

#[derive(Clone)]
pub struct PoseProxy(mpsc::Sender<Command>);
impl PoseProxy {

    pub async fn inference_start(&self) {
        self.0.send(Command::InferenceStart).await.unwrap();
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
    output: Option<broadcast::Sender<PoseEventSink>>,
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
                output: None,
                engine,
                is_running: false
            },
            PoseProxy(sender),
        )
    }

    fn handle_message(&mut self, msg: Command) {
        match msg {
            Command::InferenceStart => {
                self.engine.inference_start("/dev/video0");
                self.is_running = true;
            }
            Command::InferenceStop => {
                self.engine.inference_end();
                self.is_running = false;
            }
            /// Request a direct connection to the pose output
            Command::RequestDataStream { respond_to } => todo!()
        }
    }

    pub async fn run(mut self) {
        tokio::task::block_in_place(move || {
            loop {
                // Generate a pose estimation
                if self.is_running {
                    println!("pose - inference");

                    let pose = self.engine.inference_step();
                    println!("{:?}", pose);

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
