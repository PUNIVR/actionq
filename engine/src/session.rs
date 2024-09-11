use tokio::sync::{mpsc, oneshot};

use crate::hpe::EngineProxy;

pub enum Command {
    SessionStart,
    SessionEnd,
}

struct Session {
    receiver: mpsc::Receiver<Command>,
    in_progress: bool,

    engine_proxy: EngineProxy,
}

#[derive(Clone)]
pub struct SessionProxy(mpsc::Sender<Command>);
impl SessionProxy {}

impl Session {
    fn instantiate(engine_proxy: &EngineProxy) -> (Self, SessionProxy) {
        let (tx, rx) = mpsc::channel(100);
        (
            Self {
                receiver: rx,
                in_progress: false,
                engine_proxy: engine_proxy.clone(),
            },
            SessionProxy(tx),
        )
    }

    async fn run(mut self) {
        println!("Session - run");
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                Command::SessionStart => {
                    println!("Session: session start");
                }
                Command::SessionEnd => {
                    println!("Session: session end");
                }
            }
        }
    }
}

pub fn run_session(engine_proxy: &EngineProxy) -> SessionProxy {
    let (session, proxy) = Session::instantiate(engine_proxy);
    tokio::spawn(session.run());
    proxy
}
