use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use firestore::{
    paths, FirestoreDb, FirestoreListenerTarget, FirestoreTempFilesListenStateStorage,
};

use actionq_motion::ParameterDescriptor;
use actionq_common::{
    JetsonRequest, JetsonExerciseRequest,
    firestore::{
        ExerciseTemplate
    }
};

// Run an async block in place
#[macro_export]
macro_rules! sync_async {
    ($rt:expr, $f:expr) => {
        $rt.block_on(async { $f })
    };
}

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    request: Command,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExerciseReps {
    exercise_id: String,
    num_repetitions: u32,
    /// Optional runtime parameters
    parameters: Option<HashMap<String, f32>>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Command {
    SessionStart {
        exercises: Vec<ExerciseReps>,
        save: bool,
    },
    SetPlayState {
        running: bool,
    },
    SessionEnd,
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
    async fn update_command(&self, request: Command) {
        let _: Document = self
            .db
            .fluent()
            .update()
            .fields(paths!(Document::request))
            .in_col("jetson")
            .document_id(&self.patient_id)
            .object(&Document { request })
            .execute()
            .await
            .unwrap();
    }

    /// Stop an exercise
    pub async fn set_play_state(&self, running: bool) {
        self.update_command(Command::SetPlayState { running }).await;
    }

    // Run single exercise
    pub async fn run_exercise(&self, exercise_id: String, num_repetitions: u32, parameters: Option<HashMap<String, f32>>) {
        self.update_command(Command::SessionStart { 
            save: false, 
            exercises: vec![
                ExerciseReps {
                    exercise_id, num_repetitions, parameters
                }
            ]
        }).await;
    }

    // Stop a session
    pub async fn session_stop(&self) {
        self.update_command(Command::SessionEnd).await;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExerciseParameter {
    name: String,
    description: String,
    default: f32
}

#[derive(Debug, Serialize, Deserialize)]
struct Exercise {
    name: String,
    description: String,
    parameters: Vec<ParameterDescriptor>,
    gif: String,
    fsm: String,
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
        gif_uri: &str,
        parameters: Vec<ParameterDescriptor>
    ) {
        let obj = Exercise {
            name: name.into(),
            description: description.into(),
            fsm: fsm.into(),
            gif: gif_uri.into(),
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
        let _: Exercise = self
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

