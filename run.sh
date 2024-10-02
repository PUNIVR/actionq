#!/bin/bash

LD_LIBRARY_PATH=$LD_LIBRARY_PATH:prepose/libs
RUST_LOG=none,prepare_engine=trace,prepose=trace,motion=trace ./target/debug/prepare_engine


