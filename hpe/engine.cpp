#include <NvInferRuntime.h>
#include <NvInferRuntimeBase.h>
#include <cstdio>
#include <cstdlib>
#include <iostream>
#include <opencv2/opencv.hpp>
#include <opencv2/core/cuda.hpp>
#include <opencv2/cudaimgproc.hpp>
#include <engine.hpp>
#include <fstream>
#include "NvInfer.h"

class Logger : public nvinfer1::ILogger {
  void log(Severity severity, const char *msg) noexcept override {
    std::cout << msg << "\n";
  }
};

struct TRTEngine {

  Logger logger;

  nvinfer1::IRuntime* runtime;
  nvinfer1::ICudaEngine* cuda_engine;
  nvinfer1::IExecutionContext* ctx;

  nvinfer1::Dims input_dim, output_dim;
  std::vector<std::string> io_tensor_names;
};

static TRTEngine g_engine;
static cv::VideoCapture g_camera;
static cv::Mat g_current_frame;
static int g_current_frame_number;
static pose g_current_pose;
static bool g_running;

/*
static struct {

  cv::VideoCapture camera;
  int frame_number;
  cv::Mat frame;

  nvinfer1::IRuntime* runtime;
  nvinfer1::ICudaEngine* cuda_engine;
  nvinfer1::IExecutionContext* ctx;

} g;
*/

void print_shape_dims(const nvinfer1::Dims& dims) {
  for (int i = 0; i < dims.nbDims; i++)
    std::cout << "\t" << dims.d[i] << "\n";
}

TRTEngine trt_engine_crate(const std::string& filepath) {
  
  std::ifstream file(filepath, std::ios::binary | std::ios::ate);
  std::streamsize size = file.tellg();
  file.seekg(0, std::ios::beg);

  std::vector<char> buffer(size);
  file.read(buffer.data(), size);

  Logger logger;
  
  nvinfer1::IRuntime* runtime = nvinfer1::createInferRuntime(logger);
  nvinfer1::ICudaEngine* engine = runtime->deserializeCudaEngine(buffer.data(), buffer.size());
  nvinfer1::IExecutionContext* ctx = engine->createExecutionContext();

  nvinfer1::Dims input_dim, output_dim;
  std::vector<std::string> io_tensor_names{};

  // Analyze the IO tensors of the model
  for (int i = 0; i < engine->getNbIOTensors(); i++) {
    const auto name = engine->getIOTensorName(i);
    std::cout << "found tensor: " << name << "\n";
    io_tensor_names.push_back(name);

    const auto itype = engine->getTensorIOMode(name);
    const auto shape = engine->getTensorShape(name);
    const auto dtype = engine->getTensorDataType(name);
  
    if (itype == nvinfer1::TensorIOMode::kINPUT) {
      input_dim = shape;
      std::cout << "input tensor of shape: \n";
      print_shape_dims(shape);

    } else if (itype == nvinfer1::TensorIOMode::kOUTPUT) {
      output_dim = shape;
      std::cout << "output tensor of shape: \n";
      print_shape_dims(shape);
    }
  }

  return TRTEngine {
    logger,
    runtime,
    engine,
    ctx,
    input_dim,
    output_dim
  };
}

void trt_engine_inference(TRTEngine& engine, const cv::cuda::GpuMat& frame) {

  // Preprocessing
  cv::cuda::GpuMat RGBframe;
  cv::cuda::cvtColor(frame, RGBframe, cv::COLOR_BGR2RGB);

  std::vector<cv::cuda::GpuMat> input_tmp{std::move(frame)};
  std::vector<std::vector<cv::cuda::GpuMat>> input{std::move(input_tmp)};
  std::vector<std::vector<std::vector<float>>> output{};

  // Where to put input data
  cudaStream_t inferenceCudaStream;
  cudaStreamCreate(&inferenceCudaStream);

  engine.ctx->setInputShape(engine.io_tensor_names[0].c_str(), engine.input_dim);
  engine.ctx->setTensorAddress(engine.io_tensor_names[0].c_str(), input.data());

  engine.ctx->enqueueV3(inferenceCudaStream);

  cudaStreamSynchronize(inferenceCudaStream);
  cudaStreamDestroy(inferenceCudaStream);
}


void initialize(char* model_name) {
  std::cout << "initialize\n";
  cv::namedWindow("Camera", cv::WINDOW_AUTOSIZE);
  g_engine = trt_engine_crate("plans/yolov8s-pose.trt");
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
  //std::cout << "inference_step: " << g_current_frame_number << "\n";
  
  if (g_running == true) {
    g_current_frame_number += 1;

    g_camera >> g_current_frame;

    cv::cuda::GpuMat gpu_frame;
    gpu_frame.upload(g_current_frame);
    trt_engine_inference(g_engine, gpu_frame);

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
