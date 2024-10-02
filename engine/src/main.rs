use tokio;
use tracing_subscriber::EnvFilter;

mod network;
mod pose;
mod session;

fn setup_tracing() {

    //let filter = EnvFilter::

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)  // enable everything
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .with_file(false)                       // Display source code file paths
        .with_line_number(false)                // Display source code line numbers
        .with_thread_ids(true)                  // Display the thread ID an event was recorded on
        .with_target(false)                     // Don't display the event's target (module path)
        .init();                                // sets this to be the default, global collector for this application.
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    setup_tracing();

    let (pose, pose_receiver) = pose::run_human_pose_estimator();
    let session = session::run_session(&pose, pose_receiver);
    network::run_websocket_server("0.0.0.0:3666", &session, &pose)
        .await
        .expect("Server - error");

    Ok(())
}
