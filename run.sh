#!/bin/bash

LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/local/lib/
RUST_LOG=none,prepare_engine=trace,prepose=trace,motion=trace ./target/debug/prepare_engine


