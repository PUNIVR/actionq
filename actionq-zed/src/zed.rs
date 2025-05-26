use actionq_common::*;

#[cxx::bridge]
mod ffi {

    struct Vec2 { pub x: f32, pub y: f32 }
    struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

    struct Pose {
        /// Screen-space coordinates of the person's keypoints relative to the left camera
        pub keypoints_2d: Vec<Vec2>,
        /// Global coordinates of the person's keypoints in 3D space
        pub keypoints_3d: Vec<Vec3>
    }

    struct CaptureData {
        pub height: usize,
        pub width: usize,
        pub frame: Vec<u8>,
        pub pose: Pose
    }

    // Interface functions
    unsafe extern "C++" {
        include!("actionq-zed/include/zed.hh");

        /// Initialize ZED camera and AI models
        unsafe fn initialize();
        /// Returns Human Pose of current frame
        unsafe fn extract() -> CaptureData;
        /// Close everything
        unsafe fn finish();
    }
}

/// Initialize ZED camera and AI models
pub fn initialize() {
    unsafe { ffi::initialize(); }
}

/// Returns Human Pose of current frame
pub fn extract() -> CaptureData {
    let output: ffi::CaptureData = unsafe { ffi::extract() };
    output.into()
}

/// Close everything
pub fn finish() {
    unsafe { ffi::finish(); }
}

/// Convert Vec2 from ffi to glam
impl Into<glam::Vec2> for ffi::Vec2 {
    fn into(self) -> glam::Vec2 {
        glam::Vec2::new(self.x, self.y)
    }
} 

/// Convert Vec3 from ffi to glam
impl Into<glam::Vec3> for ffi::Vec3 {
    fn into(self) -> glam::Vec3 {
        glam::Vec3::new(self.x, self.y, self.z)
    }
}

/// Convert CaptureData from ffi to crate
impl Into<CaptureData> for ffi::CaptureData {
    fn into(self) -> CaptureData {
        assert!(self.frame.len() == self.width * self.height * 4);
        CaptureData {
            frame: self.frame,
            resolution: Resolution {
                h: self.height,
                w: self.width,
            },
            pose: Pose {
                kp2d: skeleton_map_body_coco18(
                    &self.pose.keypoints_2d.iter()
                        .map(|v| glam::Vec2::new(v.x, v.y))
                        .collect()
                    ),
                kp3d: skeleton_map_body_coco18(
                    &self.pose.keypoints_3d.iter()
                        .map(|v| glam::Vec3::new(v.x, v.y, v.z))
                        .collect()
                    )
            }
        }
    }
}

