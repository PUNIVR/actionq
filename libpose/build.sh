#!/bin/bash

mkdir -p extern && cd extern

# Download jetson-inference with correct version
git clone --recursive https://github.com/dusty-nv/jetson-inference.git
git checkout 01a3958 # for L4T R32.7.1
cd jetson-inference

# Build jetson-inference and jetson-utils
mkdir -p build && cd build
cmake ..
sudo make install
make -j$(nproc)
sudo ldconfig

# Build libpose
cd ../../../
mkdir -p build && cd build
cmake ..
make -j$(nproc)