#!/bin/bash
set -e

# Compile eBPF program
echo "🚀 Compiling eBPF program..."
cargo build --release --target bpfel-unknown-none -Z build-std -p ebpf-monitor

# Compile User-space loader
echo "🚀 Compiling User-space loader..."
cargo build --release -p ebpf-monitor-user

echo "✅ Build complete!"
