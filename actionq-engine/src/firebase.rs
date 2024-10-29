use tokio::sync::oneshot::{Sender, Receiver};
use firestore::{FirestoreListenEvent, FirestoreDb, FirestoreMemListenStateStorage, FirestoreListenerTarget};
use serde::{Serialize, Deserialize};
use crate::session::SessionProxy;
use crate::network::Requests;

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    #[serde(alias = "_firestore_id")]
    doc_id: Option<String>,
    #[serde(flatten)]
    request: Requests
}

#[tracing::instrument(skip_all)]
async fn on_event(event: FirestoreListenEvent, session: SessionProxy) {
    match event {
        FirestoreListenEvent::DocumentChange(ref doc_change) => {
            tracing::trace!("handling document change");
            if let Some(doc) = &doc_change.document {

                // Try deserialize the document into a command
                let doc = match FirestoreDb::deserialize_doc_to::<Document>(doc) {
                    Ok(doc) => doc,
                    Err(e) => {
                        tracing::error!("cannot deserialize request: {}", e);
                        return;
                    }
                };

                tracing::trace!("request: {:?}", doc);

                // Send request to session
                match doc.request {
                    Requests::SessionStart => {
                        tracing::info!("request: session start");
                        session.session_start().await;
                    }
                    Requests::SessionEnd => {
                        tracing::info!("request: session end");
                        session.session_end().await;
                    },
                    Requests::ExerciseStart { exercise_id } => {
                        tracing::info!("request: exercise start");
                        session.exercise_start(exercise_id).await;
                    },
                    Requests::ExerciseEnd => {
                        tracing::info!("request: exercise end");
                        session.exercise_end().await;
                    },
                    _ => unimplemented!()
                }
            }
        }
        _ => { 
            tracing::info!("handling other events...");
        }
    }

}

#[tracing::instrument(skip(session, kill))]
async fn listen_commands(patient_id: String, database_id: String, session: SessionProxy, kill: Receiver<()>) {

    tracing::info!("connecting to firestore database");
    let db = FirestoreDb::new(database_id)
        .await.expect("unable to connect to firestore database");

    // Add commands document for the patient
    tracing::info!("reseting commands document for patient");
    let doc = Document { doc_id: None, request: Requests::SessionStart };
    let doc_res: Document = db.fluent().update().in_col("commands")
        .document_id(&patient_id).object(&doc)
        .execute().await.expect("unable to add patient's command document");

    // Listen to collection's mutations
    tracing::info!("creating listener for commands of patient");
    let mut listener = db.create_listener(FirestoreMemListenStateStorage::new())
        .await.expect("unable to create listener for document changes");

    db.fluent().select().by_id_in("commands").batch_listen([patient_id])
        .add_target(FirestoreListenerTarget::new(78), &mut listener)
        .expect("unable to attach listener to commands of patient");
    
    // Start background listener as a tokio task
    listener.start(move |event| {
        let session = session.clone();
        async move {
            on_event(event, session).await;
            Ok(())
        }
    }).await.expect("unable to listen to changes");

    // Wait for exit signal
    kill.await.unwrap();

    listener.shutdown().await
        .expect("unable to shutdown listener");
}

pub fn run_event_listener(patient_id: &str, database_id: &str, session: &SessionProxy) -> Sender<()> {
    let (kill_tx, kill_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(listen_commands(patient_id.into(), database_id.into(), session.clone(), kill_rx));
    kill_tx
}
