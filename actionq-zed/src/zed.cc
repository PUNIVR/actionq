#include <actionq-zed/include/zed.hh>
#include <actionq-zed/src/zed.rs.h>

static sl::Camera g_zed;
static sl::BodyTrackingParameters g_detection_params;
static sl::BodyTrackingRuntimeParameters g_body_params;

/// Initialize ZED camera and AI models
void initialize() {

	sl::InitParameters init_parameters;
    init_parameters.camera_resolution = sl::RESOLUTION::AUTO;
    init_parameters.depth_mode = sl::DEPTH_MODE::NEURAL;
    init_parameters.coordinate_units = sl::UNIT::METER;
	init_parameters.sdk_verbose = true;

	// Open the camera
    auto state = g_zed.open(init_parameters);
    if (state != sl::ERROR_CODE::SUCCESS) {
    	exit(-1);
    }

    // Different model can be chosen, optimizing the runtime or the accuracy
    g_detection_params.detection_model = sl::BODY_TRACKING_MODEL::HUMAN_BODY_MEDIUM;
    // Body format
    g_detection_params.body_format = sl::BODY_FORMAT::BODY_18;
    // track detects object across time and space
    g_detection_params.enable_tracking = true;
    // Optimize the person joints position, requires more computations
    g_detection_params.enable_body_fitting = true;

	// If you want to have object tracking you need to enable positional tracking first
    if (g_detection_params.enable_tracking)
        g_zed.enablePositionalTracking();

	state = g_zed.enableBodyTracking(g_detection_params);
    if (state != sl::ERROR_CODE::SUCCESS) {
		finish();
		exit(-1);
    }

    // For outdoor scene or long range, the confidence should be lowered to avoid missing detections (~20-30)
    // For indoor scene or closer range, a higher confidence limits the risk of false positives and increase the precision (~50+)
    g_body_params.detection_confidence_threshold = 40;
}

/// Returns Human Pose of current frame
CaptureData extract() {
	CaptureData result{};

	sl::RuntimeParameters runtime_params;
	runtime_params.measure3D_reference_frame = sl::REFERENCE_FRAME::WORLD;

	sl::Bodies bodies;
	if (g_zed.grab(runtime_params) == sl::ERROR_CODE::SUCCESS) {

		// Get backbuffer
		static sl::Mat backbuffer;
		g_zed.retrieveImage(backbuffer, sl::VIEW::LEFT);

		result.height = backbuffer.getHeight();
		result.width = backbuffer.getWidth();

		// Copy backbuffer
		sl::uchar1* data = backbuffer.getPtr<sl::uchar1>();
		result.frame.reserve(result.width * result.height * 4);
		for (int i = 0; i < result.width * result.height * 4; ++i)
			result.frame.push_back(data[i]);

		// Cannot use this, size is not updated
		//std::memcpy(result.frame.data(), backbuffer.getPtr<sl::uchar1>(), result.frame.size());

		//printf("buffer resolution: %d x %d, channels: %d\n", 
		//	backbuffer.getWidth(), backbuffer.getHeight(), backbuffer.getChannels());

		// Get skeleton
		g_zed.retrieveBodies(bodies, g_body_params);

		if (!bodies.is_new)
			return result;

		// Skip computation if there are no bodies
		if (bodies.body_list.empty())
			return result;

		auto body = bodies.body_list.front();

		// Store skeleton global positions
		for (int i = 0; i < body.keypoint.size(); ++i) {
			auto &kp = body.keypoint[i];
			result.pose.keypoints_3d.push_back(Vec3 {
				kp.x, kp.y, kp.z
			});
		}

		// Store skeleton 2D positions
		for (int i = 0; i < body.keypoint_2d.size(); ++i) {
			auto &kp = body.keypoint_2d[i];
			result.pose.keypoints_2d.push_back(Vec2 {
				kp.x, kp.y
			});
		}
	}

	return result;	
}

/// Close everything
void finish() {
	g_zed.close();
}


