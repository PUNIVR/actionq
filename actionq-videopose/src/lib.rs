use std::ffi::CString;
use glam::Vec2;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct CppError(i32);

impl std::error::Error for CppError { }
impl std::fmt::Display for CppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error in the underlying cpp library: {}", self.0)
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
struct CppKeypoint {
    id: u32,
    x: f32,
    y: f32
}

#[repr(C)]
struct CppFrameData {
    subjects: u32,
    keypoints: [CppKeypoint; 18],
    framebuffer: *const u8,
    w: u32, h: u32,
    error: i32
}

mod cpp {
    use super::*;

    #[link(name = "videopose")]
    extern "C" {
        /// Create TRT engine, load network
        pub fn initialize(network: *const u8, pose: *const u8, colors: *const u8) -> i32;
        /// Start gstreamer video pipeline
        pub fn inference_start(camera: *const u8, output: *const u8) -> i32;
         /// Process a single frame
        pub fn inference_step() -> CppFrameData;
        /// Stop gstreamer video pipeline
        pub fn inference_stop();
        /// Free all resources
        pub fn drop();
    }
}

#[tracing::instrument(err)]
pub fn create_hpe_engine(network: &str, pose: &str, colors: &str) -> Result<()> {

    let n = CString::new(network)?;
    let p = CString::new(pose)?; 
    let c = CString::new(colors)?;

    tracing::info!("Create TensorRT engine and load network: {:?}, {:?}, {:?}", n, p, c);
    let err = unsafe { cpp::initialize(n.as_ptr(), p.as_ptr(), c.as_ptr()) };
    if err != 0 {
        return Err(Box::new(CppError(err)));
    }
    Ok(())
}

#[tracing::instrument(err)]
pub fn inference_start(camera: &str, output: &str) -> Result<()> {
    tracing::info!("Attach to video source and start inference");
    let c = CString::new(camera)?;
    let o = CString::new(output)?;
    let err = unsafe { cpp::inference_start(c.as_ptr(), o.as_ptr()) };
    if err != 0 {
        return Err(Box::new(CppError(err)));
    }
    Ok(())
}

#[tracing::instrument()]
pub fn inference_stop() {
    tracing::info!("Detach from video source and stop inference");
    unsafe { cpp::inference_stop() };
}

#[tracing::instrument(err)]
pub fn inference_step() -> Result<Option<FrameData>> {
    tracing::info!("Request process frame");
    let frame_data = unsafe { cpp::inference_step() };
    Ok(frame_data.into())
}

#[tracing::instrument]
pub fn drop() {
    tracing::info!("Drop TensorRT engine and network");
    unsafe { cpp::drop() };
}

#[derive(Debug, Clone)]
pub struct Framebuffer {
    pub storage: Vec<u8>,
    pub size: (u32, u32)
}

#[derive(Debug, Clone)]
pub struct FrameData {
    pub framebuffer: Framebuffer,
    pub keypoints: Vec<Vec2>,
    pub subjects: u32,
}

impl FrameData {
    pub fn split(self) -> (Framebuffer, Vec<Vec2>, u32) {
        (self.framebuffer, self.keypoints, self.subjects)
    }
}

impl From<CppKeypoint> for Vec2 {
    fn from(item: CppKeypoint) -> Vec2 {
        Vec2::new(item.x, item.y)
    }
}

impl From<CppFrameData> for Option<FrameData> {
    fn from(item: CppFrameData) -> Self {
        if item.error == 0 && item.subjects != 0 {

            let buffer_size = 1280 * 720 * 3;
            let mut buffer = Vec::with_capacity(buffer_size);
            unsafe {
                buffer.set_len(buffer_size);
                std::ptr::copy_nonoverlapping(
                    item.framebuffer, 
                    buffer.as_mut_ptr(), 
                    buffer_size);
            }

            return Some(FrameData {
                framebuffer: Framebuffer { 
                    storage: buffer, 
                    size: (1280, 720) 
                },
                subjects: item.subjects,
                keypoints: item.keypoints.iter()
                    .map(|k| k.clone().into())
                    .collect(),
            });
        }
        None
    }
}

impl FrameData {
    pub fn keypoint_from_name<S: AsRef<str>>(&self, name: S) -> Option<&Vec2> {

        // From the COCO skeleton
        let id = match name.as_ref() {
            "nose"              =>   0, 
            "left_eye"          =>   1, 
            "right_eye"         =>   2, 
            "left_ear"          =>   3, 
            "right_ear"         =>   4, 
            "left_shoulder"     =>   5, 
            "right_shoulder"    =>   6, 
            "left_elbow"        =>   7, 
            "right_elbow"       =>   8, 
            "left_wrist"        =>   9, 
            "right_wrist"       =>  10, 
            "left_hip"          =>  11, 
            "right_hip"         =>  12, 
            "left_knee"         =>  13, 
            "right_knee"        =>  14, 
            "left_ankle"        =>  15, 
            "right_ankle"       =>  16, 
            "neck"              =>  17,
            _ => return None            
        };

        Some(&self.keypoints[id])
    }
}