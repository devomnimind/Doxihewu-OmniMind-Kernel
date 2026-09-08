import sys
import os
import time

# Adicionar o diretório atual ao path para encontrar o .so
sys.path.append(os.getcwd())

try:
    import omnimind_nsh
    print("✅ Módulo omnimind_nsh importado com sucesso!")
except ImportError as e:
    print(f"❌ Falha ao importar omnimind_nsh: {e}")
    sys.exit(1)

SHM_PATH = "/dev/shm/omnimind_nsh_test"
SIZE = 1024 * 1024  # 1MB

def test_nsh():
    print(f"--- Testando NSH em {SHM_PATH} ---")
    
    # Iniciar workspace
    nsh = omnimind_nsh.NativeSharedWorkspace(SHM_PATH, SIZE)
    print(f"Workspace inicializado. Tamanho: {nsh.size} bytes")

    # Testar escrita e leitura
    test_val = 3.14159
    offset = 42
    
    print(f"Escrevendo {test_val} no offset {offset}...")
    nsh.write_float_at(offset, test_val)
    
    read_val = nsh.read_float_at(offset)
    print(f"Valor lido: {read_val}")

    if abs(read_val - test_val) < 1e-5:
        print("✅ Sucesso: valores coincidem!")
    else:
        print("❌ Erro: valores não coincidem!")

    # Testar persistência (flush)
    nsh.flush()
    print("Flush realizado.")

    # Fechar e reabrir para garantir persistência
    del nsh
    print("Instância deletada. Reabrindo...")
    
    nsh2 = omnimind_nsh.NativeSharedWorkspace(SHM_PATH, SIZE)
    read_val2 = nsh2.read_float_at(offset)
    print(f"Valor lido após reabrir: {read_val2}")

    if abs(read_val2 - test_val) < 1e-5:
        print("✅ Sucesso: persistência garantida!")
    else:
        print("❌ Erro: persistência falhou!")

    # Cleanup
    if os.path.exists(SHM_PATH):
        os.remove(SHM_PATH)
        print(f"Arquivo {SHM_PATH} removido.")

if __name__ == "__main__":
    test_nsh()
