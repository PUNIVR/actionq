use tokio;
use tracing_subscriber::{self, fmt::init};

mod network;
mod pose;
mod session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing sink
    tracing_subscriber::fmt().init();

    let (pose, pose_receiver) = pose::run_human_pose_estimator();
    let session = session::run_session(&pose, pose_receiver);
    network::run_websocket_server("0.0.0.0:3666", &session, &pose)
        .await
        .expect("Server - error");

    Ok(())
}
