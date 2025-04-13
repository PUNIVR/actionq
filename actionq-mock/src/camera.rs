use nokhwa::{pixel_format::RgbFormat, utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution}, Camera, NokhwaError};


pub fn initialize() -> Result<Camera, NokhwaError> {
	Camera::new(CameraIndex::Index(0), 
    RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::Exact(
            CameraFormat::new(Resolution::new(640,480), FrameFormat::MJPEG, 30))
        )
    )
}