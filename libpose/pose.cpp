
#include <jetson-utils/videoSource.h>
#include <jetson-utils/videoOptions.h>
#include <jetson-utils/URI.h>

#include <jetson-inference/poseNet.h>
#include <pose.hpp>

static const uint32_t g_overlay_flags = poseNet::OverlayFlagsFromStr("keypoints");
static videoSource* g_camera = nullptr;
static poseNet* g_network = nullptr;

API void initialize() {
    g_network = poseNet::Create(NETWORK_PATH, POSE_PATH, COLORS_PATH);
    if (!g_network) {
        printf("error - unable to create network\n");
    }
}

API void inference_start(const char* video) {
    g_camera = videoSource::Create(video, 0, nullptr);
    if (!g_camera) {
        printf("error - unable to open camera\n");
    }
}

API void inference_end() {
    SAFE_DELETE(g_camera);
}

API pose_data inference_step() {
    pose_data result = {0};

    // Get frame from image
    uchar3* image = nullptr;
    if (!g_camera->Capture(&image, 1000)) {
        if (!g_camera->IsStreaming()) {
            printf("error - unable to capture frame\n");
            result.error = 1;
            return result;
        }
    }

    // Get pose from network
    std::vector<poseNet::ObjectPose> poses;
    if (!g_network->Process(image, g_camera->GetWidth(), g_camera->GetHeight(), poses, g_overlay_flags)) {
        printf("error - unable to process frame for body pose\n");
        result.error = 2;
        return result;
    }

    // Construct pose stimation result
    result.detected_subjects = poses.size();
    result.detected_kps = poses[0].Keypoints.size();

    for (int i = 0; i < result.detected_kps; i++) {
        const auto& kp = poses[0].Keypoints[i];
        result.kps[i].x = kp.x;
        result.kps[i].y = kp.y;
    }

    return result;
}

API void shutdown() {
    SAFE_DELETE(g_network);
}

