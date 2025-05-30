use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Keypoints names of the COCO18 body
pub const COCO18: [&str; 18] = [
    "nose",
    "neck",
    "right_shoulder",
    "right_elbow",
    "right_wrist",
    "left_shoulder",
    "left_elbow",
    "left_wrist",
    "right_hip",
    "right_knee",
    "right_ankle",
    "left_hip",
    "left_knee",
    "left_ankle",
    "right_eye",
    "left_eye",
    "right_ear",
    "left_ear",
];

pub type SkeletonMap<T> = HashMap<String, T>;
pub type Skeleton<T> = Vec<T>;

/// Assign keypoints names to a skeleton using a reference body
pub fn skeleton_map_body_coco18<T: Clone>(skeleton: &Skeleton<T>) -> SkeletonMap<T> {
    skeleton
        .iter()
        .enumerate()
        .map(|(i, pos)| (COCO18[i].to_string(), pos.clone()))
        .collect()
}

pub type SkeletonMap2D = SkeletonMap<glam::Vec2>;
pub type SkeletonMap3D = SkeletonMap<glam::Vec3>;

pub type Skeleton2D = Skeleton<glam::Vec2>;
pub type Skeleton3D = Skeleton<glam::Vec3>;

#[derive(Debug, Clone)]
pub struct Resolution {
    pub w: usize,
    pub h: usize,
}

/// The captured pose of a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    /// Screen-space coordinates of the person's keypoints relative to the left camera
    pub kp2d: SkeletonMap2D,
    /// Global coordinates of the person's keypoints in 3D space
    pub kp3d: SkeletonMap3D,
}

/// A captured frame and 2D/3D skeleton
#[derive(Debug, Clone)]
pub struct CaptureData {
    pub resolution: Resolution,
    pub frame: Vec<u8>,
    pub pose: Pose,
}

/// Describe an exercise request
#[derive(Debug, Serialize, Deserialize)]
pub struct JetsonExerciseRequest {
    /// Overwrites default parameters of the exercise
    pub parameters: Option<HashMap<String, f32>>,
    /// Number of repetitions to do to consider this exercise completed
    pub num_repetitions: u32,
    /// Id of the exercise
    pub exercise_id: String,
}

/// Possible requests for the Jetson
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JetsonRequest {
    /// Starts a new session, if one is already in progress then connect to that
    /// session without starting a new one
    SessionStart {
        /// Optional patient Id, if present the session and exercise data is stored in his database
        /// section (Used for the RSA mode). Otherwise they are stored in the Jetson root patient_id.
        patient_id: Option<String>,
        /// The exercises that must be completed in this session, they are in order of execution
        exercises: Vec<JetsonExerciseRequest>,
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

/// Possible responses of the Jetson
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JetsonResponse { }

/// Contains all objects used only in the database
pub mod firebase {
    use super::{JetsonRequest, JetsonResponse};
    use serde::{Serialize, Deserialize};
    
    /// Wrapper for a Jetson request or response with an unique Id to prevent idempotency removal of events
    #[derive(Debug, Serialize, Deserialize)]
    pub struct IdempotencyWrap<T> {
        //pub dedup_id: String,
        #[serde(flatten)]
        pub inner: T,
    }


    /// Descriptor of an exercise
    /// NOTE: The default parameters are not included, are they are present in the FSM script
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ExerciseTemplate {
        /// Id of the exercise
        pub name: String,
        /// Description of the exercise
        pub description: String,
        /// Code of the FSM running the exercise
        pub fsm: String,
    }

    /// Jetson interface to handle events and errors
    #[derive(Debug, Serialize, Deserialize)]
    pub struct JetsonInterface {
        /// Buffer where requests are written by the clients
        pub request: Option<IdempotencyWrap<JetsonRequest>>,
        /// Buffer where responses are written by the jetson
        pub response: Option<JetsonResponse>
    }

    /// Descriptor of completed session of exercises
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SessionStore {
        /// Exercises to do during the session
        pub exercises: Vec<ExerciseStore>,
        /// When the session was completed
        pub timestamp: String,

    }

    /// Descriptor of a completed exercise
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ExerciseStore {
        /// Number of repetitions completed
        pub num_repetitions_done: u32,
        /// Id of the exercise
        pub exercise: String,
    }
}
