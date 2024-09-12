use tokio;
use tokio::signal::ctrl_c;

mod pose;
mod network;
mod session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (pose, pose_receiver) = pose::run_human_pose_estimator();
    let session = session::run_session(&pose, pose_receiver);
    network::run_websocket_server("0.0.0.0:3666", &session, &pose)
        .await
        .expect("Server - error");

    Ok(())
}
