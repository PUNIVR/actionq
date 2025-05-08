#include <sl/Camera.hpp>

struct CaptureData;

/// Initialize ZED camera and AI models
void initialize();

/// Returns data for current frame
CaptureData extract();

/// Close everything
void finish();


