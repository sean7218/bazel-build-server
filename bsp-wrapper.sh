#!/bin/bash
# Save both stdin and stdout of your build server

LOG_DIR="/Users/sean7218/bazel/buildserver"
mkdir -p "$LOG_DIR"

tee "$LOG_DIR/from-sourcekit.log" | RUST_LOG=debug /Users/sean7218/bazel/buildserver/target/debug/buildserver | tee "$LOG_DIR/to-sourcekit.log"