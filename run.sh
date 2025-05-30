#!/bin/bash

export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/local/lib/
DISPLAY=:0 RUST_LOG=none,prepare_engine=info,prepose=info,motion=info ./target/release/prepare_engine --user-id VsBYJPdvik35pFYthJgx
