#include <jetson-utils/videoSource.h>
#include <jetson-utils/videoOutput.h>
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

#define KEYPOINTS_COUNT 18
using Keypoint = poseNet::ObjectPose::Keypoint;

static struct {
    uint32_t overlay_flags;
    videoSource* camera;
    poseNet* network;
} g;

#define FB_SIZE 921600 // 1280x720
#define FB_CHANNELS 3
#define FB_BYTES (FB_SIZE * FB_CHANNELS)

static uchar3 LAST_FB_DATA[FB_SIZE]; 

struct Frame {
    /// Subjects present in the scene
    uint32_t subjects;
    /// All keypoints (even not detected ones) for the first subject
    Keypoint keypoints[KEYPOINTS_COUNT];
    /// Pointer to the last processed framedata
    uchar3* framebuffer;
    /// Size of the framebuffer
    uint32_t w, h;
    /// Error code
    int error;
};

extern "C" {
    /// Create TRT engine, load network
    API int initialize(const char* network_path, const char* pose_path, const char* colors_path);
    /// Start gstreamer video pipeline
    API int inference_start(const char* cam, const char* output);
    /// Process a single frame
    API Frame inference_step();
    /// Stop gstreamer video pipeline
    API void inference_stop();
    /// Free all resources
    API void drop();
}

API int initialize(const char* network_path, const char* pose_path, const char* colors_path) {

    // TODO: find a way to handel this logger better
    Log::SetLevel(Log::Level::DEBUG);

    // Initialize framebuffer to zeros
    memset(LAST_FB_DATA, 0, FB_BYTES);

    // Load network and create TRT engine
    g.overlay_flags = poseNet::OverlayFlagsFromStr("keypoints,links");
    g.network = poseNet::Create(network_path, pose_path, colors_path);
    return (g.network) ? 0 : 1;
}

API int inference_start(const char* cam, const char* output) {
    g.camera = videoSource::Create(cam, 0, nullptr);
    return (g.camera) ? 0 : 1;
}

API void inference_stop() {
    SAFE_DELETE(g.camera);
}

API Frame inference_step() {
    Frame result = {0};

    uint32_t w = g.camera->GetWidth();
    uint32_t h = g.camera->GetHeight();

    // Get frame from image
    int status = 0;
    uchar3* framebuffer = nullptr;
    if (!g.camera->Capture(&framebuffer, &status)) {
        if (!g.camera->IsStreaming()) {
            result.error = 1;
            return result;
        }
    }

    // Get pose from network
    std::vector<poseNet::ObjectPose> poses;
    if (!g.network->Process(framebuffer, w, h, poses, g.overlay_flags)) {
        result.error = 2;
        return result;
    }

    // Construct pose estimation result
    result.subjects = poses.size();
    if (result.subjects != 0) {
        const auto& keypoints = poses[0].Keypoints;
        for (int i = 0; i < keypoints.size(); i++)
            result.keypoints[i] = keypoints[i];
    }

    // Copy framebuffer to CPU memory
    cudaMemcpy(LAST_FB_DATA, framebuffer, FB_BYTES, cudaMemcpyDeviceToHost);
    result.framebuffer = LAST_FB_DATA;

    return result;
}

API void drop() {
    SAFE_DELETE(g.network);
}