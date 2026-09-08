import sys
import os
import time
import numpy as np

# Adicionar caminhos para os módulos
sys.path.append(os.getcwd())

try:
    import omnimind_metrics
    print("✅ Módulo omnimind_metrics importado!")
except ImportError as e:
    print(f"❌ Falha ao importar omnimind_metrics: {e}")
    sys.exit(1)

def test_allmini():
    dim = 384
    print(f"--- Testando normalização e similaridade para dim={dim} (allmini) ---")
    
    # Criar vetores aleatórios
    v1 = [np.random.rand() for _ in range(dim)]
    v2 = [v1[i] + (np.random.rand() * 0.1) for i in range(dim)] # v1 com ruído
    
    # Normalizar em Rust
    start_norm = time.time()
    v1_norm = omnimind_metrics.normalize_l2(v1)
    v2_norm = omnimind_metrics.normalize_l2(v2)
    end_norm = time.time()
    
    print(f"Normalização concluída em {(end_norm - start_norm)*1000:.4f} ms")
    
    # Verificar norma
    norm1 = np.linalg.norm(v1_norm)
    norm2 = np.linalg.norm(v2_norm)
    print(f"Norma V1: {norm1:.6f}, Norma V2: {norm2:.6f}")
    
    if abs(norm1 - 1.0) < 1e-5 and abs(norm2 - 1.0) < 1e-5:
        print("✅ Sucesso: Vetores normalizados corretamente!")
    else:
        print("❌ Erro: Normalização falhou!")

    # Similaridade de cosseno
    start_sim = time.time()
    sim = omnimind_metrics.cosine_similarity(v1_norm, v2_norm)
    end_sim = time.time()
    
    print(f"Similaridade de Cosseno: {sim:.6f}")
    print(f"Cálculo de similaridade concluído em {(end_sim - start_sim)*1000:.4f} ms")

    # Comparar com NumPy
    sim_np = np.dot(v1_norm, v2_norm) / (np.linalg.norm(v1_norm) * np.linalg.norm(v2_norm))
    print(f"Similaridade NumPy: {sim_np:.6f}")
    
    if abs(sim - sim_np) < 1e-5:
        print("✅ Sucesso: Similaridade coincide com NumPy!")
    else:
        print("❌ Erro: Similaridade diverge!")

if __name__ == "__main__":
    test_allmini()
