use std::os::raw::{c_char, c_int};
use std::ffi::CString;

use glam::Vec2;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct LibKeypoint {
    x: f32,
    y: f32
}

#[repr(C)]
struct LibPoseData {
    detected_subjects: c_int,
    detected_kps: c_int,
    kps: [LibKeypoint; 18],
    error: c_int
}

#[derive(Debug, Clone)]
pub struct PoseData {
    pub subjects: usize,
    pub valid_kps: usize,
    pub kps: Vec<Vec2>
}

impl From<LibKeypoint> for Vec2 {
    fn from(item: LibKeypoint) -> Vec2 {
        Vec2::new(
            item.x, 
            item.y
        )
    }
}

impl From<LibPoseData> for Option<PoseData> {
    fn from(item: LibPoseData) -> Self {

        if item.error == 0 {
            return Some(PoseData {
                subjects: item.detected_subjects as usize,
                valid_kps: item.detected_kps as usize,
                kps: item.kps.iter()
                    .map(|k| k.clone().into())
                    .collect(),
            });
        }
        None
    }
}


impl PoseData {
    pub fn keypoint_from_name<S: AsRef<str>>(&self, name: S) -> Option<&Vec2> {

        // From the COCO skeleton
        let id = match name.as_ref() {
            "nose" =>              0, 
            "left_eye" =>          1, 
            "right_eye" =>         2, 
            "left_ear" =>          3, 
            "right_ear" =>         4, 
            "left_shoulder" =>     5, 
            "right_shoulder" =>    6, 
            "left_elbow" =>        7, 
            "right_elbow" =>       8, 
            "left_wrist" =>        9, 
            "right_wrist" =>      10, 
            "left_hip" =>         11, 
            "right_hip" =>        12, 
            "left_knee" =>        13, 
            "right_knee" =>       14, 
            "left_ankle" =>       15, 
            "right_ankle" =>      16, 
            "neck" =>             17,
            _ => return None            
        };

        Some(&self.kps[id])
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

pub struct PoseEstimator;
impl PoseEstimator {

    pub fn new(network: &str, pose: &str, colors: &str) -> Self {
        let network = CString::new(network).unwrap();
        let pose = CString::new(pose).unwrap();
        let colors = CString::new(colors).unwrap();
        
        unsafe {        
            initialize(
                network.as_ptr(), 
                pose.as_ptr(), 
                colors.as_ptr()
            ); 
        }
        Self
    }

    pub fn inference_start(&mut self, video: &str) {
        let video = CString::new(video).unwrap();
        unsafe { 
            inference_start(video.as_ptr()) 
        }
    }

    pub fn inference_step(&mut self) -> Option<PoseData> {
        unsafe {
            let pose = inference_step();
            pose.into()
        }
    }

    pub fn inference_end(&mut self) {
        unsafe {
            inference_end()
        }
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
            "../network/colors.txt"
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