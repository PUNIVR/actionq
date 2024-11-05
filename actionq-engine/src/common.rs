use serde::{Deserialize, Serialize};

/// Describes how many repetition for exercise
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestExerciseReps {
    pub exercise_id: String,
    pub num_repetitions: u32,
}

/// Possible requests from the client
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    /// Starts a new session, if one is already in progress then connect to that
    /// session without starting a new one
    SessionStart {
        /// The exercises that must be completed in this session, they are in order of execution
        exercises: Vec<RequestExerciseReps>,
        /// True if the engine should save the exercise execution log into the database
        /// at the end of this session
        save: bool
    },
    /// Pause and Resume current exercise running
    SetPlayState { running: bool },
    /// End the current session in progress
    SessionEnd,
    /// Close all connections
    CloseAll,
}