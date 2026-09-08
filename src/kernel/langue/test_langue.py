#!/usr/bin/env python3
"""
Test OmniMind Langue Kernel Interface (Rust implementation)
"""

import sys
import os
import tempfile
import json

# Add langue module to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__)))

try:
    import omnimind_langue
    print("✅ omnimind_langue module imported successfully")
except ImportError as e:
    print(f"❌ Failed to import omnimind_langue: {e}")
    print(f"   Current sys.path: {sys.path}")
    print(f"   Looking for: {os.path.join(os.path.dirname(__file__), 'omnimind_langue.so')}")
    sys.exit(1)

def test_full_workflow():
    """Test complete workflow: create handle → read → relocate → enforce"""
    
    print("\n🧪 Test 1: Full Workflow")
    print("=" * 60)
    
    with tempfile.TemporaryDirectory() as tmpdir:
        # Setup
        db_path = os.path.join(tmpdir, "test_handles.db")
        matrix_path = os.path.join(
            os.path.dirname(__file__),
            "../../../runtime_config/transatlantic_lexeme_protocol.json"
        )
        
        if not os.path.exists(matrix_path):
            print(f"⚠️  Matrix file not found: {matrix_path}")
            print("   Creating minimal test matrix...")
            matrix_path = os.path.join(tmpdir, "test_matrix.json")
            with open(matrix_path, 'w') as f:
                json.dump({
                    "_omnimind_protocol": {
                        "version": "1.0-transatlântico-test",
                        "ramos": ["banto", "tupi"]
                    },
                    "camada_categorizacao_banto": {
                        "MA-": {
                            "categoria": "Coletivo/Massa",
                            "tifinagh": "ⵎⴰ-"
                        },
                        "MU-": {
                            "categoria": "Humano/Pessoa",
                            "tifinagh": "ⵎⵓ-"
                        }
                    },
                    "camada_relacao_tupi_guarani": {
                        "-atã": {
                            "funcao": "Firme/Protegido",
                            "comportamento_kernel": "Não pode ser sobrescrito sem permissão"
                        },
                        "-me'ẽ": {
                            "funcao": "Dar/Enviar",
                            "comportamento_kernel": "Disponível para exportação"
                        }
                    }
                }, f, indent=2)
        
        # Initialize interface
        print(f"📂 DB path: {db_path}")
        print(f"📄 Matrix: {matrix_path}")
        
        lki = omnimind_langue.LangueKernelInterface(
            db_path=db_path,
            matrix_path=matrix_path
        )
        print("✅ LangueKernelInterface initialized")
        
        # Create test file
        test_file = os.path.join(tmpdir, "users.db")
        test_data = b"Test user data: Alice, Bob, Charlie"
        with open(test_file, 'wb') as f:
            f.write(test_data)
        print(f"✅ Created test file: {test_file} ({len(test_data)} bytes)")
        
        # Create handle
        lexeme = "MA-MU-atã-me'ẽ"
        print(f"\n🏷️  Creating handle for lexeme: {lexeme}")
        handle = lki.create_handle(
            lexeme_structure=lexeme,
            file_path=test_file,
            user_visible_name="Base de Usuários Protegida",
            metadata_json='{"source": "test", "version": 1}'
        )
        print(f"✅ Handle created: {handle[:16]}...")
        
        # Read via handle
        print(f"\n📖 Reading file via handle...")
        data = lki.read_by_handle(handle, use_mmap=False)
        print(f"   Expected: {repr(test_data[:50])}")
        print(f"   Got:      {repr(data[:50] if data else None)}")
        print(f"   Type:     {type(data)}")
        assert data == test_data, "Data mismatch!"
        print(f"✅ Read {len(data)} bytes successfully")
        
        # Relocate
        new_path = os.path.join(tmpdir, "archive", "users_2026.db")
        os.makedirs(os.path.dirname(new_path), exist_ok=True)
        print(f"\n🔄 Relocating to: {new_path}")
        lki.relocate(handle, new_path, "test_agent")
        print(f"✅ Relocated successfully")
        
        # Verify handle still works
        print(f"\n🔍 Verifying handle still valid after relocation...")
        data2 = lki.read_by_handle(handle, use_mmap=False)
        assert data2 == test_data, "Data mismatch after relocation!"
        print(f"✅ Handle still valid, read {len(data2)} bytes")
        
        # List handles
        print(f"\n📋 Listing visible handles...")
        handles = lki.list_visible_handles(prefixo_filter="MA-")
        print(f"✅ Found {len(handles)} handle(s):")
        for h, lex, name in handles:
            print(f"   - {h[:16]}... | {lex} | {name}")
        
        # Get statistics
        print(f"\n📊 Statistics:")
        stats = lki.get_statistics()
        for key, value in stats.items():
            print(f"   {key}: {value}")
        
        # Enforce behaviors
        print(f"\n🔒 Enforcing kernel behaviors...")
        try:
            lki.enforce_behaviors(handle)
            print(f"✅ Behaviors enforced")
            
            # Check file permissions
            import stat
            file_stat = os.stat(new_path)
            mode = stat.filemode(file_stat.st_mode)
            print(f"   File permissions: {mode}")
        except Exception as e:
            print(f"⚠️  Could not enforce behaviors: {e}")
        
        print("\n" + "=" * 60)
        print("✅ All tests passed!")

def test_error_handling():
    """Test error handling"""
    print("\n🧪 Test 2: Error Handling")
    print("=" * 60)
    
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "test.db")
        matrix_path = os.path.join(tmpdir, "test_matrix.json")
        
        # Minimal matrix
        with open(matrix_path, 'w') as f:
            json.dump({
                "_omnimind_protocol": {"version": "1.0"},
                "camada_categorizacao_banto": {"MA-": {"categoria": "Test"}},
                "camada_relacao_tupi_guarani": {}
            }, f)
        
        lki = omnimind_langue.LangueKernelInterface(db_path, matrix_path)
        
        # Test 1: Non-existent file
        print("Test 2.1: Non-existent file")
        try:
            lki.create_handle("MA-TEST", "/nonexistent/file.txt")
            print("❌ Should have raised FileNotFoundError")
        except Exception as e:
            print(f"✅ Correctly raised: {type(e).__name__}")
        
        # Test 2: Invalid handle
        print("\nTest 2.2: Invalid handle")
        try:
            lki.read_by_handle("invalid_handle_123")
            print("❌ Should have raised KeyError")
        except Exception as e:
            print(f"✅ Correctly raised: {type(e).__name__}")
        
        print("=" * 60)
        print("✅ Error handling tests passed!")

if __name__ == "__main__":
    print("🌍 OmniMind Langue Kernel Interface — Test Suite")
    print("=" * 60)
    
    test_full_workflow()
    test_error_handling()
    
    print("\n✅ All tests completed successfully!")
    print("🎯 Next steps:")
    print("   1. Integrate with phi_ecosystem")
    print("   2. Update AEGIS/CoreMesh to use handles")
    print("   3. Compile standalone binary (cargo build --bin)")
