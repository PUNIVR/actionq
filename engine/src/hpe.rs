use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub struct PoseData {}

// Receiver for HPE data
pub struct PoseEventSink(broadcast::Receiver<PoseData>);

pub enum Command {
    StartInference,

    /// Request a direct connection to the pose output
    RequestDataStream {
        respond_to: oneshot::Sender<PoseEventSink>,
    },
}

#[derive(Clone)]
pub struct EngineProxy(mpsc::Sender<Command>);
impl EngineProxy {
    pub async fn start_inference(&self) {
        self.0.send(Command::StartInference).await.unwrap();
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
struct Engine {
    receiver: mpsc::Receiver<Command>,
    output: Option<broadcast::Sender<PoseEventSink>>,
}

impl Engine {
    pub fn instantiate() -> (Self, EngineProxy) {
        let (sender, receiver) = mpsc::channel(100);
        (
            Engine {
                receiver,
                output: None,
            },
            EngineProxy(sender),
        )
    }

    pub async fn run(mut self) {
        println!("Engine - run");
    }
}

pub fn run_human_pose_estimator() -> EngineProxy {
    let (engine, proxy) = Engine::instantiate();
    tokio::spawn(engine.run());
    proxy
}
