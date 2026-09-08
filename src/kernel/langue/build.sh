#!/bin/bash
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

echo "🚀 Building OmniMind Langue Kernel Interface (Rust)"
echo "📦 Project root: $PROJECT_ROOT"

cd "$PROJECT_ROOT/src/kernel/langue"

# Build release binary
echo "🔨 Compiling Rust → Python extension..."
cargo build --release

# Copy .so to Python-visible location
SO_FILE="target/release/libomnimind_langue.so"
if [ -f "$SO_FILE" ]; then
    cp "$SO_FILE" "$PROJECT_ROOT/src/kernel/langue/omnimind_langue.so"
    echo "✅ Built: omnimind_langue.so"
    ls -lh "$PROJECT_ROOT/src/kernel/langue/omnimind_langue.so"
else
    echo "❌ Build failed: $SO_FILE not found"
    exit 1
fi

echo ""
echo "✅ Build complete!"
echo "📍 Module location: src/kernel/langue/omnimind_langue.so"
echo ""
echo "🧪 To test:"
echo "   cd $PROJECT_ROOT"
echo "   PYTHONPATH=src/kernel/langue python src/kernel/langue/test_langue.py"
