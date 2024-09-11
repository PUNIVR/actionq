#pragma once

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
    keypoint kps[20];
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