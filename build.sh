#!/bin/bash

# CompressO CLI build script for Linux and macOS

set -e

cd "$(dirname "$0")"

echo "Building CompressO CLI..."
cargo build --release

echo ""
echo "Build complete!"
echo "Binary location: target/release/compresso"
