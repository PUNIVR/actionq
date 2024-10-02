use std::ffi::CString;
use std::os::raw::{c_char, c_int};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Keypoint {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
struct LibPoseData {
    detected_subjects: c_int,
    detected_kps: c_int,
    kps: [Keypoint; 20],
    error: c_int,
}

#[derive(Debug, Clone)]
pub struct PoseData {
    pub subjects: usize,
    pub valid_kps: usize,
    pub kps: [Keypoint; 20],
}

impl From<LibPoseData> for Option<PoseData> {
    fn from(item: LibPoseData) -> Self {
        if item.error == 0 {
            return Some(PoseData {
                subjects: item.detected_subjects as usize,
                valid_kps: item.detected_kps as usize,
                kps: item.kps,
            });
        }
        None
    }
}

#[link(name = "pose")]
extern "C" {
    /// Create TRT engine, load network
    fn initialize(network: *const c_char, pose: *const c_char, colors: *const c_char);
    /// Attach to videocamera, prepare memory
    fn inference_start(video: *const c_char);
    /// Get frame from videocamera, process frame using network, return pose
    fn inference_step() -> LibPoseData;
    /// Detach from videocamera
    fn inference_end();
    /// Close everything
    fn shutdown();
}

#[derive(Debug)]
pub struct PoseEstimator;
impl PoseEstimator {
    pub fn new(network: &str, pose: &str, colors: &str) -> Self {
        let network = CString::new(network).unwrap();
        let pose = CString::new(pose).unwrap();
        let colors = CString::new(colors).unwrap();
        unsafe {
            initialize(network.as_ptr(), pose.as_ptr(), colors.as_ptr());
        }
        Self
    }

    pub fn inference_start(&mut self, video: &str) {
        let video = CString::new(video).unwrap();
        unsafe { inference_start(video.as_ptr()) }
    }

    pub fn inference_step(&mut self) -> Option<PoseData> {
        unsafe {
            let pose = inference_step();
            pose.into()
        }
    }

    pub fn inference_end(&mut self) {
        unsafe { inference_end() }
    }
}

impl Drop for PoseEstimator {
    fn drop(&mut self) {
        unsafe {
            shutdown();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end() {
        let mut pose = PoseEstimator::new(
            "../network/pose_resnet18_body.onnx",
            "../network/human_pose.json",
            "../network/colors.txt",
        );

        // Ex1
        pose.inference_start("/dev/video0");
        for _ in 0..10 {
            let pose = pose.inference_step();
            println!("{:?}", pose);
        }
        pose.inference_end();

        // Ex2
        pose.inference_start("/dev/video0");
        for _ in 0..10 {
            let pose = pose.inference_step();
            println!("{:?}", pose);
        }
        pose.inference_end();
    }
}

