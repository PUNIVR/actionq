#![allow(dead_code, unused_imports)]

use firestore::{FirestoreListenEvent, FirestoreDb, FirestoreMemListenStateStorage, FirestoreListenerTarget, paths};
use tokio::sync::mpsc::{Sender, Receiver};

use crate::session::SessionProxy;
use crate::common::{Request, RequestExerciseReps};

/// Data definitions inside of firebase
mod model {
    use serde::{Serialize, Deserialize};
    use crate::common::{Request, RequestExerciseReps};

    /// Exercise definition
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Exercise {
        pub name: String,
        pub description: String,
        pub gif: String,
        pub fsm: String
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum JetsonState { Offline, Listening, Running }

    /// Jetson commands interface
    #[derive(Debug, Serialize, Deserialize)]
    pub struct JetsonInterface {
        pub state: JetsonState,
        pub request: Option<Request>
    }
}

/// Utility struct to interact with the firestore database
struct Firestore {
    pub patient_id: String,
    pub db: FirestoreDb
}

impl Firestore {
    /// Create a new firestore database connection
    pub async fn new(google_progect_id: &str, patient_id: &str) -> Self {
        Self { 
            patient_id: patient_id.to_owned(),
            db: FirestoreDb::new(google_progect_id).await
                .expect("unable to connect to firebase")
        }
    }

    /// Get an exercise by it's Id
    pub async fn get_exercise(&self, exercise_id: &str) -> Option<model::Exercise> {
        self.db.fluent().select()
            .by_id_in("exercises")
            .obj()
            .one(exercise_id)
            .await.unwrap()
    }

    /// Set the state of the Jetson
    pub async fn set_jeston_state(&self, state: model::JetsonState) {
        let object = model::JetsonInterface { state, request: None };
        self.db.fluent().update()
            .fields(paths!(model::JetsonInterface::state))
            .in_col("jetson")
            .document_id(&self.patient_id)
            .object(&object)
            .execute::<model::JetsonInterface>()
            .await.unwrap();
    }
}

/// All accepted commands for Firebase
#[derive(Debug)]
pub enum FirebaseCommand {
    GetExerciseDefinition {
        respond_to: tokio::sync::oneshot::Sender<Option<model::Exercise>>,
        /// Id of the requested exercise
        exercise_id: String
    }
}

#[derive(Debug)]
pub struct FirebaseProxy(pub tokio::sync::mpsc::Sender<FirebaseCommand>);
impl FirebaseProxy {
    pub async fn get_exercise(&self, exercise_id: &str) -> Option<model::Exercise> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.0.send(FirebaseCommand::GetExerciseDefinition {
            respond_to: tx, exercise_id: exercise_id.to_owned()
        }).await.unwrap();
        rx.await.unwrap()
    }
}

#[tracing::instrument(skip_all)]
async fn on_event(event: FirestoreListenEvent, session: SessionProxy) {
    match event {
        FirestoreListenEvent::DocumentChange(ref doc_change) => {
            tracing::trace!("handling document change");
            if let Some(doc) = &doc_change.document {

                // Try deserialize the document into a command
                let doc = match FirestoreDb::deserialize_doc_to::<model::JetsonInterface>(doc) {
                    Ok(doc) => doc,
                    Err(e) => {
                        tracing::error!("cannot deserialize request: {}", e);
                        return;
                    }
                };

                tracing::trace!("request: {:?}", doc);

                // Send request to session
                if let Some(request) = doc.request {
                    match request {
                        Request::SessionStart { exercises, save } => {
                            tracing::info!("request: session start");
                            session.session_start(exercises, save).await;
                        }
                        Request::SessionEnd => {
                            tracing::info!("request: session end");
                            session.session_end().await;
                        },
                        Request::SetPlayState { running } => {
                            tracing::info!("request: set play state (running -> {})", running);
                            session.set_play_state(running).await;
                        },
                        _ => unimplemented!()
                    }
                }
            }
        }
        _ => { 
            tracing::info!("handling other events...");
        }
    }

}

#[tracing::instrument(skip(session, cmds))]
pub async fn listen_commands(patient_id: &str, database_id: &str, session: SessionProxy, mut cmds: Receiver<FirebaseCommand>) {

    tracing::info!("connecting to firestore database");
    let firestore = Firestore::new(&database_id, &patient_id).await;

    // Add commands document for the patient
    tracing::info!("reseting commands document for patient");
    let doc = model::JetsonInterface { request: Some(Request::SessionEnd), state: model::JetsonState::Listening };
    let doc_res: model::JetsonInterface = firestore.db.fluent().update().in_col("jetson")
        .document_id(&patient_id).object(&doc)
        .execute().await.expect("unable to add patient's command document");

    // Listen to collection's mutations
    tracing::info!("creating listener for commands of patient");
    let mut listener = firestore.db.create_listener(FirestoreMemListenStateStorage::new())
        .await.expect("unable to create listener for document changes");

    firestore.db.fluent().select().by_id_in("jetson").batch_listen([patient_id])
        .add_target(FirestoreListenerTarget::new(78), &mut listener)
        .expect("unable to attach listener to commands of patient");
    
    // Notify that we are listening for commands
    tracing::info!("setting jetson state to 'listening'");
    firestore.set_jeston_state(model::JetsonState::Listening).await;

    // Start background listener as a tokio task
    listener.start(move |event| {
        let session = session.clone();
        async move {
            on_event(event, session).await;
            Ok(())
        }
    }).await.expect("unable to listen to changes");

    // Handle other commands
    while let Some(cmd) = cmds.recv().await {
        match cmd {
            FirebaseCommand::GetExerciseDefinition { respond_to, exercise_id } => {
                let exercise = firestore.get_exercise(&exercise_id).await;
                tracing::trace!("retreived exercise: {:?}", exercise);
                respond_to.send(exercise)
                    .expect("unable to respond");            
            }
        }
    }

    // Notify that we stoppend listening for commands
    tracing::info!("setting jetson state to 'offline'");
    firestore.set_jeston_state(model::JetsonState::Offline).await;
    listener.shutdown().await
        .expect("unable to shutdown listener");
}
