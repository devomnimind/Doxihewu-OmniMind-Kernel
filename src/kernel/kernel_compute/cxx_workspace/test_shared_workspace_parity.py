import time
import numpy as np
import omnimind_workspace_cxx

def test_workspace_initialization():
    config = omnimind_workspace_cxx.WorkspaceConfig()
    config.embedding_dim = 256
    config.max_history_size = 1000

    workspace = omnimind_workspace_cxx.SharedWorkspaceCore(config)
    assert workspace is not None
    print("✅ Initialization Test Passed")
    return workspace

def test_write_read_module_state(workspace):
    module_name = "test_module_A"
    # Create a random numpy array of dimension 256
    test_embedding = np.random.rand(256).astype(np.float32).tolist()
    metadata = {"source": "test_script", "confidence": "0.99"}

    # Write to C++ Core
    workspace.write_module_state(module_name, test_embedding, metadata)

    # Read from C++ Core
    retrieved_embedding = workspace.read_module_state(module_name)
    retrieved_metadata = workspace.read_module_metadata(module_name)

    # Validate precision and integrity
    np.testing.assert_allclose(test_embedding, retrieved_embedding, rtol=1e-5)
    assert retrieved_metadata["source"] == "test_script"
    assert retrieved_metadata["confidence"] == "0.99"
    
    # Check history
    history = workspace.get_module_history(module_name, last_n=5)
    assert len(history) == 1
    assert history[0].module_name == module_name
    assert history[0].cycle == 0
    np.testing.assert_allclose(history[0].embedding, test_embedding, rtol=1e-5)

    print("✅ Write/Read Integrity Test Passed")

def test_normalization(workspace):
    module_name = "test_module_B"
    # Test oversized embedding (should be truncated to 256)
    oversized_embedding = np.random.rand(300).astype(np.float32).tolist()
    workspace.write_module_state(module_name, oversized_embedding)
    retrieved = workspace.read_module_state(module_name)
    assert len(retrieved) == 256
    np.testing.assert_allclose(oversized_embedding[:256], retrieved, rtol=1e-5)
    
    # Test undersized embedding (should be padded with 0.0 to 256)
    module_name_c = "test_module_C"
    undersized_embedding = np.random.rand(100).astype(np.float32).tolist()
    workspace.write_module_state(module_name_c, undersized_embedding)
    retrieved_c = workspace.read_module_state(module_name_c)
    assert len(retrieved_c) == 256
    np.testing.assert_allclose(undersized_embedding, retrieved_c[:100], rtol=1e-5)
    assert all(x == 0.0 for x in retrieved_c[100:])

    print("✅ Normalization (Truncation & Padding) Test Passed")

def run_all_tests():
    print("--- Running Golden Tests for SharedWorkspaceCore (C++) ---")
    workspace = test_workspace_initialization()
    test_write_read_module_state(workspace)
    test_normalization(workspace)
    print("--- All Tests Passed Successfully! ---")

if __name__ == "__main__":
    run_all_tests()
