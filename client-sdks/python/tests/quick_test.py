#!/usr/bin/env python3
"""
Quick test script to verify the new Python SDK tests.
This script checks if all new test modules can be imported and run.
"""

import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'tests'))

def test_imports():
    """Test if all new modules can be imported."""
    print("Testing imports...")
    print("-" * 60)
    
    try:
        print("Importing test_intelligent_search...", end=" ")
        import test_intelligent_search
        print("OK")
        
        print("Importing test_discovery...", end=" ")
        import test_discovery
        print("OK")
        
        print("Importing test_file_operations...", end=" ")
        import test_file_operations
        print("OK")
        
        print("\n[OK] All imports successful!")
        return True
        
    except Exception as e:
        print(f"\n[FAIL] Import failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_syntax():
    """Test Python syntax of new test files."""
    print("\nTesting syntax...")
    print("-" * 60)
    
    test_files = [
        'test_intelligent_search.py',
        'test_discovery.py',
        'test_file_operations.py'
    ]
    
    tests_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'tests')
    
    all_ok = True
    for filename in test_files:
        filepath = os.path.join(tests_dir, filename)
        if os.path.exists(filepath):
            try:
                print(f"Checking {filename}...", end=" ")
                with open(filepath, 'r', encoding='utf-8') as f:
                    code = f.read()
                compile(code, filename, 'exec')
                print("OK")
            except SyntaxError as e:
                print(f"FAIL - Syntax error: {e}")
                all_ok = False
            except Exception as e:
                print(f"FAIL - Error: {e}")
                all_ok = False
        else:
            print(f"FAIL - {filename} not found")
            all_ok = False
    
    if all_ok:
        print("\n[OK] All syntax checks passed!")
    else:
        print("\n[FAIL] Some syntax checks failed!")
    
    return all_ok

def test_models():
    """Test if all required models are available."""
    print("\nTesting model availability...")
    print("-" * 60)
    
    try:
        from models import (
            IntelligentSearchRequest,
            IntelligentSearchResponse,
            SemanticSearchRequest,
            SemanticSearchResponse,
            ContextualSearchRequest,
            ContextualSearchResponse,
            MultiCollectionSearchRequest,
            MultiCollectionSearchResponse,
        )
        
        print("[OK] IntelligentSearchRequest")
        print("[OK] IntelligentSearchResponse")
        print("[OK] SemanticSearchRequest")
        print("[OK] SemanticSearchResponse")
        print("[OK] ContextualSearchRequest")
        print("[OK] ContextualSearchResponse")
        print("[OK] MultiCollectionSearchRequest")
        print("[OK] MultiCollectionSearchResponse")
        
        print("\n[OK] All required models are available!")
        return True
        
    except Exception as e:
        print(f"\n[FAIL] Model import failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_client_methods():
    """Test if all required client methods exist."""
    print("\nTesting client methods...")
    print("-" * 60)
    
    try:
        from client import VectorizerClient
        
        methods = [
            'intelligent_search',
            'semantic_search',
            'contextual_search',
            'multi_collection_search',
            'discover',
            'filter_collections',
            'score_collections',
            'expand_queries',
            'get_file_content',
            'list_files_in_collection',
            'get_file_summary',
            'get_file_chunks_ordered',
            'get_project_outline',
            'get_related_files',
            'search_by_file_type',
        ]
        
        for method_name in methods:
            if hasattr(VectorizerClient, method_name):
                print(f"[OK] {method_name}")
            else:
                print(f"[FAIL] {method_name} - NOT FOUND!")
                return False
        
        print("\n[OK] All required client methods exist!")
        return True
        
    except Exception as e:
        print(f"\n[FAIL] Client method check failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    """Main test function."""
    print("=" * 60)
    print("QUICK TEST FOR NEW PYTHON SDK FEATURES")
    print("=" * 60)
    
    results = []
    
    # Run tests
    results.append(("Syntax", test_syntax()))
    results.append(("Imports", test_imports()))
    results.append(("Models", test_models()))
    results.append(("Client Methods", test_client_methods()))
    
    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    for test_name, result in results:
        status = "[PASSED]" if result else "[FAILED]"
        print(f"{test_name:20} {status}")
    
    print(f"\nTotal: {passed}/{total} tests passed")
    
    if passed == total:
        print("\n[OK] All quick tests passed!")
        print("The new test files are ready to run.")
        print("\nTo run the full test suite:")
        print("  ./pytest.sh    (Linux/Mac)")
        print("  pytest.bat     (Windows)")
        return 0
    else:
        print("\n[FAIL] Some quick tests failed!")
        print("Please fix the issues above before running the full test suite.")
        return 1

if __name__ == "__main__":
    sys.exit(main())

