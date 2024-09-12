use tokio;
use tokio::signal::ctrl_c;

mod pose;
mod network;
mod session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pose_proxy = pose::run_human_pose_estimator();
    let session_proxy = session::run_session(&pose_proxy);

    network::run_websocket_server("0.0.0.0:3666", &session_proxy, &pose_proxy)
        .await
        .expect("Server - error");

    Ok(())
}
