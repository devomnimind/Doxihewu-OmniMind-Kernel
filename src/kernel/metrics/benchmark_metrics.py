import sys
import os
import time
import random
import numpy as np

# Adicionar caminhos para os módulos
sys.path.append(os.getcwd())
sys.path.append("/opt/omnimind")

from src.metrics.neutrosophic_metrics import NeutrosophicLogic, NeutrosophicQuadruple

try:
    import omnimind_metrics
    print("✅ Módulo omnimind_metrics importado com sucesso!")
except ImportError as e:
    print(f"❌ Falha ao importar omnimind_metrics: {e}")
    sys.exit(1)

def benchmark():
    num_alternatives = 10000 # 100k
    print(f"--- Gerando {num_alternatives} alternativas aleatórias ---")
    
    # Gerar dados
    alts_raw = [[random.random() for _ in range(4)] for _ in range(num_alternatives)]
    alts_objects = [NeutrosophicQuadruple(*v) for v in alts_raw]
    
    ideal_pos = NeutrosophicQuadruple(1.0, 1.0, 0.0, 0.0)
    ideal_neg = NeutrosophicQuadruple(0.0, 0.0, 1.0, 1.0)
    
    ideal_pos_vec = [1.0, 1.0, 0.0, 0.0]
    ideal_neg_vec = [0.0, 0.0, 1.0, 1.0]

    print("\n--- Benchmark: quadruple_topsis ---")
    
    # Python
    start_py = time.time()
    res_py = NeutrosophicLogic.quadruple_topsis(alts_objects, ideal_pos, ideal_neg)
    end_py = time.time()
    py_time = end_py - start_py
    print(f"Python: {py_time:.6f} segundos")
    
    # Rust
    start_rs = time.time()
    res_rs = omnimind_metrics.quadruple_topsis(alts_raw, ideal_pos_vec, ideal_neg_vec)
    end_rs = time.time()
    rs_time = end_rs - start_rs
    print(f"Rust:   {rs_time:.6f} segundos")
    
    speedup = py_time / rs_time if rs_time > 0 else float('inf')
    print(f"\n🚀 Speedup: {speedup:.2f}x")

    # Validar resultados
    diff = abs(res_py[0] - res_rs[0])
    if diff < 1e-5:
        print("✅ Validação: Resultados coincidem!")
    else:
        print(f"❌ Validação: Resultados divergem! Diff: {diff}")

if __name__ == "__main__":
    benchmark()
