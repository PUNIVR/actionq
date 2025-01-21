#![allow(dead_code, unused_imports)]

use tokio;
use tracing_subscriber::EnvFilter;
use std::io::Read;

mod pose;
mod session;
mod ui;
mod firebase;
mod common;

use firebase::FirebaseProxy;

fn setup_tracing() {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    setup_tracing();

    //let fsm = exercise::JsonExercise::simple();
    //let fsm_string = serde_json::to_string(&fsm);
    //println!("{:?}", fsm_string);
    //return Ok(());

    // Channel for UI messages
    let (ui_tx, ui_rx) = tokio::sync::mpsc::channel(100);
    let ui_proxy = ui::UiProxy(ui_tx);

    // Channel for firebase messages
    let (firebase_tx, mut firebase_rx) = tokio::sync::mpsc::channel(100);
    let firebase = FirebaseProxy(firebase_tx);

    // Move the tokio runtime to a different thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
        let rt_enter = rt.enter();
        rt.block_on(async {

            let (pose, pose_receiver) = pose::run_human_pose_estimator();
            let session = session::run_session(&pose, pose_receiver, ui_proxy, firebase);

            // This is the control interface of the system
            firebase::listen_commands("5Y7GXWsn2eJKn7tq6l7I", "uvc-unisco", session, firebase_rx)
                .await;

            //let server = network::run_websocket_server("0.0.0.0:3666", &session, &pose)
            //    .await.expect("Server - error");
        });
    });

    ui::run_ui_blocking(ui_rx);
    Ok(())
}
