cmake_minimum_required(VERSION 3.13)
project(hpe_trt_exec)

# Set C++ version and optimization level
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Ofast -DNDEBUG -Wno-deprecated-declarations")

find_package(CUDA REQUIRED)
find_package(OpenCV REQUIRED)

add_library(hpe_trt_core SHARED pose.cpp)
set_target_properties(hpe_trt_core PROPERTIES OUTPUT_NAME "pose")

target_include_directories(hpe_trt_core PUBLIC ${OpenCV_INCLUDE_DIRS} ${CUDA_INCLUDE_DIRS} "include")
target_link_libraries(hpe_trt_core ${OpenCV_LIBS} ${CUDA_LIBRARIES})

# We need jetson-inference and jetson-utils from https://github.com/dusty-nv/jetson-inference
target_link_directories(hpe_trt_core PUBLIC "libs")
target_link_libraries(hpe_trt_core jetson-inference jetson-utils)

# As a test
add_executable(hpe_trt_infer inference.cpp)
set_target_properties(hpe_trt_infer PROPERTIES OUTPUT_NAME "rt_infer_webcam")
target_link_libraries(hpe_trt_infer hpe_trt_core)