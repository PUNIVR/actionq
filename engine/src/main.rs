use tokio;
use tokio::signal::ctrl_c;

mod hpe;
mod network;
mod session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine_proxy = hpe::run_human_pose_estimator();
    let session_proxy = session::run_session(&engine_proxy);

    network::run_websocket_server("0.0.0.0:3666", &session_proxy, &engine_proxy)
        .await
        .expect("Server - error");

    Ok(())
}
