#define JOINT_COUNT 17 
#define CAMERA_DIMS 2

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

struct pose {
  float coords[CAMERA_DIMS * JOINT_COUNT];
};

extern "C" {

  /// Create TRT engine, load network
API void initialize(char* model_name);
  /// Attach to videocamera, prepare memory
API void inference_start();
  /// Get frame from videocamera, process frame using network, return pose
API pose inference_step(bool show_frame);
  /// Detach from videocamera
API void inference_end();
  /// Close everything
API void shutdown();

}
