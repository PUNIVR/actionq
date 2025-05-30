use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use firestore::{
    paths, FirestoreDb, FirestoreListenerTarget, FirestoreTempFilesListenStateStorage,
};
use uuid::Uuid;

use actionq_motion::ParameterDescriptor;
use actionq_common::{
    JetsonRequest, JetsonExerciseRequest,
    firebase::{
        IdempotencyWrap, JetsonInterface, ExerciseTemplate
    }
};

// Run an async block in place
#[macro_export]
macro_rules! sync_async {
    ($rt:expr, $f:expr) => {
        $rt.block_on(async { $f })
    };
}

// Control the session of a patient
pub struct SessionCtrl {
    patient_id: String,
    db: FirestoreDb,
}

impl SessionCtrl {
    /// Create a session controller for a specific patient
    pub async fn new(patient_id: &str, database_id: &str) -> Self {
        Self {
            patient_id: patient_id.into(),
            db: FirestoreDb::new(database_id)
                .await
                .expect("unable to connect to firestore database"),
        }
    }

    /// Modify the command document
    async fn update_command(&self, request: JetsonRequest) {
        //let dedup_id: String = format!("{}", Uuid::new_v4()); 
        let _: JetsonInterface = self
            .db
            .fluent()
            .update()
            .in_col("jetson")
            .document_id(&self.patient_id)
            .object(&JetsonInterface {
                request: Some(IdempotencyWrap::<JetsonRequest> {
                    inner: request,
          //          dedup_id
                }),
                response: None,
            })
            .execute()
            .await
            .unwrap();
    }

    /// Stop an exercise
    pub async fn set_play_state(&self, running: bool) {
        self.update_command(JetsonRequest::SetPlayState { running }).await;
    }

    // Run single exercise
    pub async fn run_exercise(&self, exercise_id: String, num_repetitions: u32, patient_id: Option<String>, 
                              parameters: Option<HashMap<String, f32>>) {
        self.update_command(JetsonRequest::SessionStart {
            patient_id,
            save: false, 
            exercises: vec![
                JetsonExerciseRequest {
                    exercise_id, num_repetitions, parameters
                }
            ]
        }).await;
    }

    // Stop a session
    pub async fn session_stop(&self) {
        self.update_command(JetsonRequest::SessionEnd).await;
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ExerciseTemplateExt {
    #[serde(flatten)]
    template: ExerciseTemplate,
    /// Extracted default parameters
    parameters: Vec<ParameterDescriptor>,
}

// Interact with the database
pub struct DatabaseCtrl {
    db: FirestoreDb,
}

impl DatabaseCtrl {
    /// Create a database controller
    pub async fn new(database_id: &str) -> Self {
        Self {
            db: FirestoreDb::new(database_id)
                .await
                .expect("unable to connect to firestore database"),
        }
    }

    /// Insert a new exercise
    pub async fn add_exercise_definition(
        &self,
        name: &str,
        description: &str,
        fsm: &str,
        parameters: Vec<ParameterDescriptor>
    ) {
        let obj = ExerciseTemplateExt {
            template: ExerciseTemplate {
                name: name.into(),
                description: description.into(),
                fsm: fsm.into(),
            },
            parameters
        };

        // Remove exercise if already present
        let _ = self
            .db
            .fluent()
            .delete()
            .from("exercises")
            .document_id(name)
            .execute()
            .await;

        // Insert new
        let _: ExerciseTemplateExt = self
            .db
            .fluent()
            .insert()
            .into("exercises")
            .document_id(name)
            .object(&obj)
            .execute()
            .await
            .expect("unable to insert new exercise definition");
    }
}

