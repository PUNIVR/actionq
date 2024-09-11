#!/bin/bash

# Download jetson-inference
mkdir -p extern && cd extern
git clone https://github.com/dusty-nv/jetson-inference.git
cd jetson-inference

# We must use the correct version L4T-R32.7.1
git checkout 01a3958
git submodule update --init

# Download only the required networks
cd tools && ./download-models.sh && cd ..

# Build jetson-inference and jetson-utils
mkdir -p build && cd build
cmake .. && make -j$(nproc)

# Install on the device
sudo make install
sudo ldconfig
