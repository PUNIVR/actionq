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
    "left_ear"
];

pub type SkeletonMap<T> = HashMap<String, T>;
pub type Skeleton<T> = Vec<T>;

/// Assign keypoints names to a skeleton using a reference body
pub fn skeleton_map_body_coco18<T: Clone>(skeleton: &Skeleton<T>) -> SkeletonMap<T> {
    skeleton.iter()
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
    pub h: usize
}

/// The captured pose of a person 
#[derive(Debug, Clone)]
pub struct Pose {
    /// Screen-space coordinates of the person's keypoints relative to the left camera
    pub keypoints_2d: Skeleton2D,
    /// Global coordinates of the person's keypoints in 3D space
    pub keypoints_3d: Skeleton3D
}

/// A captured frame and 2D/3D skeleton 
#[derive(Debug, Clone)]
pub struct CaptureData {
    pub resolution: Resolution,
    pub frame: Vec<u8>,
    pub pose: Pose
}

