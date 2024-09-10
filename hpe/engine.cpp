#include <cstdio>
#include <cstdlib>
#include <iostream>
#include <opencv2/opencv.hpp>
#include <engine.hpp>

static cv::VideoCapture g_camera;
static cv::Mat g_current_frame;
static int g_current_frame_number;
static pose g_current_pose;
static bool g_running;

void initialize(char* model_name) {
  std::cout << "initialize\n";
  cv::namedWindow("Camera", cv::WINDOW_AUTOSIZE);
  g_running = false;
}

void inference_start() {
  std::cout << "inference_start\n";

  g_camera = cv::VideoCapture(0);
  if (!g_camera.isOpened()) {
    std::cerr << "unable to open camera\n";
    exit(1);
  }

  // Desired camera's properties
  g_camera.set(cv::CAP_PROP_BUFFERSIZE, 1);
  g_camera.set(cv::CAP_PROP_FPS, 20);
  g_camera.set(cv::CAP_PROP_FOURCC, cv::VideoWriter::fourcc('M', 'J', 'P', 'G'));

  g_current_frame_number = 0;
  g_running = true;
}

pose inference_step(bool show_frame) {
  std::cout << "inference_step: " << g_current_frame_number << "\n";
  
  if (g_running == true) {
    g_current_frame_number += 1;
    g_camera >> g_current_frame;
  }

  if (show_frame)
    cv::imshow("Camera", g_current_frame);
  
  return {};
}

void inference_end() {
  std::cout << "inference_end\n";

  g_camera.release();
  g_running = false;
}

void shutdown() { }
