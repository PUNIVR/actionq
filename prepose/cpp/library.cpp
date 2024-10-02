#include <jetson-utils/videoSource.h>
#include <jetson-utils/videoOptions.h>
#include <jetson-utils/URI.h>
#include <jetson-inference/poseNet.h>

#if defined(_MSC_VER)
  //  Microsoft 
  #define API __declspec(dllexport)
#elif defined(__GNUC__)
  //  GCC
  #define API  __attribute__((visibility("default")))
#else
  #pragma warning Unknown dynamic link import/export semantics.
  #define API
#endif

#define NETWORK_PATH    "network/pose_resnet18_body.onnx"
#define POSE_PATH       "network/human_pose.json"
#define COLORS_PATH     "network/colors.txt"

struct keypoint {
    float x;
    float y;
};

struct pose_data {
    int detected_subjects;
    int detected_kps;
    keypoint kps[18];
    int error;
};

extern "C" {
    /// Create TRT engine, load network
    API void initialize(const char* network_path, const char* pose_path, const char* colors_path);
    /// Attach to videocamera, prepare memory
    API void inference_start(const char* video);
    /// Get frame from videocamera, process frame using network, return pose
    API pose_data inference_step();
    /// Detach from videocamera
    API void inference_end();
    /// Close everything
    API void shutdown();
}

static const uint32_t g_overlay_flags = poseNet::OverlayFlagsFromStr("keypoints");
static videoSource* g_camera = nullptr;
static poseNet* g_network = nullptr;

API void initialize(const char* network_path, const char* pose_path, const char* colors_path) {
    g_network = poseNet::Create(network_path, pose_path, colors_path);
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
    printf("info - inference stop\n");
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
    if (result.detected_subjects != 0) {
        result.detected_kps = poses[0].Keypoints.size();
        
        for (int i = 0; i < result.detected_kps; i++) {
            const auto& kp = poses[0].Keypoints[i];
            result.kps[i].x = kp.x;
            result.kps[i].y = kp.y;
        }
    }

    return result;
}

API void shutdown() {
    SAFE_DELETE(g_network);
}