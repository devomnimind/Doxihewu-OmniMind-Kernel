#!/usr/bin/env python3
import sys
import os
import time
import math
import shutil
from pathlib import Path

# Add parent directory to path to find project modules
PROJECT_ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(PROJECT_ROOT))

# Setup symlink for compiled Rust library so Python can import it directly
lib_dir = Path(__file__).resolve().parent / "engine" / "target" / "release"
src_so = lib_dir / "liblayered_transition_engine.so"
dest_so = Path(__file__).resolve().parent / "layered_transition_engine.so"

if src_so.exists() and not dest_so.exists():
    shutil.copy(src_so, dest_so)
    print("Symlink/Copy of compiled Rust shared library successfully created.")

try:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import layered_transition_engine
    print(f"Layered transition engine compiled Rust module loaded successfully. Version: {layered_transition_engine.__version__}")
except ImportError as e:
    print(f"FAILED to load Rust module: {e}")
    sys.exit(1)

# Let's perform benchmarks comparing Python vs Rust!

# 1. Lineage Sync Benchmark
def python_lineage_sync(cycle_count: int) -> float:
    return sum([math.sin(cycle_count / (i + 1)) for i in range(16)]) / 16.0

print("\n--- BENCHMARK 1: Lineage Sync (100,000 iterations) ---")
cycles = 100000

t0 = time.perf_counter()
for c in range(cycles):
    python_lineage_sync(c)
t_py = time.perf_counter() - t0
print(f"Python: {t_py:.5f} seconds")

t0 = time.perf_counter()
for c in range(cycles):
    layered_transition_engine.compute_lineage_sync(c)
t_rs = time.perf_counter() - t0
print(f"Rust:   {t_rs:.5f} seconds")
print(f"Speedup: {t_py / t_rs:.2f}x faster in Rust")


# 2. Euclidean Norm Benchmark
print("\n--- BENCHMARK 2: Euclidean L2 Norm (100,000 iterations, 768-dim) ---")
import numpy as np
vector = np.random.randn(768).tolist()

t0 = time.perf_counter()
for _ in range(cycles):
    np.linalg.norm(vector)
t_py = time.perf_counter() - t0
print(f"Python (NumPy): {t_py:.5f} seconds")

t0 = time.perf_counter()
for _ in range(cycles):
    layered_transition_engine.compute_vector_norm(vector)
t_rs = time.perf_counter() - t0
print(f"Rust:           {t_rs:.5f} seconds")
print(f"Speedup:        {t_py / t_rs:.2f}x faster in Rust")


# 3. Embedding Aggregation Benchmark
print("\n--- BENCHMARK 3: Embedding Aggregation (10,000 iterations, 20 vectors of 768-dim) ---")
embeddings = [np.random.randn(768).tolist() for _ in range(20)]
iterations = 10000

def python_aggregate(embs, noise_scale=0.1):
    stacked = np.array(embs)
    base_output = np.mean(stacked, axis=0)
    # add simple pseudo random noise
    noise = np.random.randn(768) * noise_scale
    output = base_output + noise
    norm = np.linalg.norm(output)
    if norm > 1e-12:
        output = output / norm
    return output.tolist()

t0 = time.perf_counter()
for _ in range(iterations):
    python_aggregate(embeddings)
t_py = time.perf_counter() - t0
print(f"Python (NumPy): {t_py:.5f} seconds")

t0 = time.perf_counter()
for _ in range(iterations):
    layered_transition_engine.aggregate_embeddings(embeddings, 0.1)
t_rs = time.perf_counter() - t0
print(f"Rust:           {t_rs:.5f} seconds")
print(f"Speedup:        {t_py / t_rs:.2f}x faster in Rust")

print("\nAll tests completed successfully!")
