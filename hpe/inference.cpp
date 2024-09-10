#include <engine.hpp>
#include <opencv2/opencv.hpp>

int main (int argc, char *argv[]) {

  initialize("yolov8s-pose.trt");

  inference_start();
  for(;;) {
    inference_step(true);
    if (cv::waitKey(1) == 27)
      break;
  }

  inference_end();
  shutdown();
  return 0;
}
