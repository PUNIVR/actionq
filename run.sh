#!/bin/bash

LD_LIBRARY_PATH=$LD_LIBRARY_PATH:prepose/libs
./target/debug/prepare_engine
ENGINE_PID=$!


