"""
Configuração e execução de testes para o SDK Python.

Este módulo fornece diferentes níveis de testes e configurações
para validar o funcionamento do SDK Python do Hive Vectorizer.
"""

import unittest
import sys
import os
import time

# Adicionar o diretório atual ao path
sys.path.append(os.path.dirname(__file__))


class TestRunner:
    """Classe para executar diferentes tipos de testes."""
    
    def __init__(self):
        self.test_results = {}
    
    def run_basic_tests(self):
        """Run basic tests."""
        print("Running Basic Tests")
        print("-" * 40)
        
        from test_simple import run_simple_tests
        start_time = time.time()
        
        try:
            success = run_simple_tests()
            duration = time.time() - start_time
            
            self.test_results['basic'] = {
                'success': success,
                'duration': duration,
                'status': 'PASSED' if success else 'FAILED'
            }
            
            return success
            
        except Exception as e:
            duration = time.time() - start_time
            print(f"ERROR: Error running basic tests: {e}")
            
            self.test_results['basic'] = {
                'success': False,
                'duration': duration,
                'status': 'ERROR',
                'error': str(e)
            }
            
            return False
    
    def run_comprehensive_tests(self):
        """Run comprehensive tests."""
        print("\nRunning Comprehensive Tests")
        print("-" * 40)
        
        try:
            from test_sdk_comprehensive import run_tests
            start_time = time.time()
            
            success = run_tests()
            duration = time.time() - start_time
            
            self.test_results['comprehensive'] = {
                'success': success,
                'duration': duration,
                'status': 'PASSED' if success else 'FAILED'
            }
            
            return success
            
        except Exception as e:
            duration = time.time() - start_time
            print(f"ERROR: Error running comprehensive tests: {e}")
            
            self.test_results['comprehensive'] = {
                'success': False,
                'duration': duration,
                'status': 'ERROR',
                'error': str(e)
            }
            
            return False
    
    def run_import_tests(self):
        """Test imports of all modules."""
        print("\nTesting Imports")
        print("-" * 40)
        
        modules_to_test = [
            'models',
            'exceptions', 
            'client',
            'cli',
            'examples'
        ]
        
        import_results = {}
        
        for module_name in modules_to_test:
            try:
                __import__(module_name)
                print(f"OK: {module_name} - OK")
                import_results[module_name] = True
            except Exception as e:
                print(f"ERROR: {module_name} - ERROR: {e}")
                import_results[module_name] = False
        
        all_imports_ok = all(import_results.values())
        
        self.test_results['imports'] = {
            'success': all_imports_ok,
            'results': import_results,
            'status': 'PASSED' if all_imports_ok else 'FAILED'
        }
        
        return all_imports_ok
    
    def run_syntax_tests(self):
        """Test syntax of all Python files."""
        print("\nTesting Syntax")
        print("-" * 40)
        
        python_files = [
            '__init__.py',
            'models.py',
            'exceptions.py',
            'client.py',
            'cli.py',
            'examples.py',
            'setup.py'
        ]
        
        syntax_results = {}
        
        for filename in python_files:
            if os.path.exists(filename):
                try:
                    with open(filename, 'r', encoding='utf-8') as f:
                        code = f.read()
                    
                    compile(code, filename, 'exec')
                    print(f"OK: {filename} - Syntax OK")
                    syntax_results[filename] = True
                    
                except SyntaxError as e:
                    print(f"ERROR: {filename} - Syntax error: {e}")
                    syntax_results[filename] = False
                except Exception as e:
                    print(f"ERROR: {filename} - Error: {e}")
                    syntax_results[filename] = False
            else:
                print(f"WARNING: {filename} - File not found")
                syntax_results[filename] = False
        
        all_syntax_ok = all(syntax_results.values())
        
        self.test_results['syntax'] = {
            'success': all_syntax_ok,
            'results': syntax_results,
            'status': 'PASSED' if all_syntax_ok else 'FAILED'
        }
        
        return all_syntax_ok

    def run_model_tests(self):
        """Run model tests."""
        print("\nTesting Models")
        print("-" * 40)

        try:
            from test_models import run_models_tests
            start_time = time.time()

            success = run_models_tests()
            duration = time.time() - start_time

            self.test_results['models'] = {
                'success': success,
                'duration': duration,
                'status': 'PASSED' if success else 'FAILED'
            }

            return success

        except Exception as e:
            print(f"ERROR: Error running model tests: {e}")

            self.test_results['models'] = {
                'success': False,
                'duration': 0,
                'status': 'ERROR',
                'error': str(e)
            }

            return False

    def run_exception_tests(self):
        """Run exception tests."""
        print("\nTesting Exceptions")
        print("-" * 40)

        try:
            from test_exceptions import run_exceptions_tests
            start_time = time.time()

            success = run_exceptions_tests()
            duration = time.time() - start_time

            self.test_results['exceptions'] = {
                'success': success,
                'duration': duration,
                'status': 'PASSED' if success else 'FAILED'
            }

            return success

        except Exception as e:
            print(f"ERROR: Error running exception tests: {e}")

            self.test_results['exceptions'] = {
                'success': False,
                'duration': 0,
                'status': 'ERROR',
                'error': str(e)
            }

            return False

    def run_validation_tests(self):
        """Run validation tests."""
        print("\nTesting Validation")
        print("-" * 40)

        try:
            from test_validation import run_validation_tests
            start_time = time.time()

            success = run_validation_tests()
            duration = time.time() - start_time

            self.test_results['validation'] = {
                'success': success,
                'duration': duration,
                'status': 'PASSED' if success else 'FAILED'
            }

            return success

        except Exception as e:
            print(f"ERROR: Error running validation tests: {e}")

            self.test_results['validation'] = {
                'success': False,
                'duration': 0,
                'status': 'ERROR',
                'error': str(e)
            }

            return False

    def run_http_client_tests(self):
        """Run HTTP client tests."""
        print("\nTesting HTTP Client")
        print("-" * 40)

        try:
            from test_http_client import run_http_client_tests
            start_time = time.time()

            success = run_http_client_tests()
            duration = time.time() - start_time

            self.test_results['http_client'] = {
                'success': success,
                'duration': duration,
                'status': 'PASSED' if success else 'FAILED'
            }

            return success

        except Exception as e:
            print(f"ERROR: Error running HTTP client tests: {e}")

            self.test_results['http_client'] = {
                'success': False,
                'duration': 0,
                'status': 'ERROR',
                'error': str(e)
            }

            return False

    def run_integration_tests(self):
        """Run integration tests."""
        print("\nTesting Integration")
        print("-" * 40)

        try:
            from test_client_integration import run_client_integration_tests
            start_time = time.time()

            success = run_client_integration_tests()
            duration = time.time() - start_time

            self.test_results['integration'] = {
                'success': success,
                'duration': duration,
                'status': 'PASSED' if success else 'FAILED'
            }

            return success

        except Exception as e:
            print(f"ERROR: Error running integration tests: {e}")

            self.test_results['integration'] = {
                'success': False,
                'duration': 0,
                'status': 'ERROR',
                'error': str(e)
            }

            return False

    def run_all_tests(self):
        """Run all tests."""
        print("RUNNING ALL PYTHON SDK TESTS")
        print("=" * 60)
        
        start_time = time.time()
        
        # Run tests in order
        tests_passed = 0
        total_tests = 9
        
        # 1. Syntax tests
        if self.run_syntax_tests():
            tests_passed += 1
        
        # 2. Import tests
        if self.run_import_tests():
            tests_passed += 1
        
        # 3. Basic tests
        if self.run_basic_tests():
            tests_passed += 1

        # 4. Model tests
        if self.run_model_tests():
            tests_passed += 1

        # 5. Exception tests
        if self.run_exception_tests():
            tests_passed += 1

        # 6. Validation tests
        if self.run_validation_tests():
            tests_passed += 1

        # 7. HTTP client tests
        if self.run_http_client_tests():
            tests_passed += 1

        # 8. Integration tests
        if self.run_integration_tests():
            tests_passed += 1

        # 9. Comprehensive tests (optional)
        try:
            if self.run_comprehensive_tests():
                tests_passed += 1
        except ImportError:
            print("\nWARNING: Comprehensive tests not available (missing dependencies)")
        
        total_duration = time.time() - start_time
        
        # Final report
        print("\n" + "=" * 60)
        print("FINAL TEST REPORT")
        print("=" * 60)
        
        print(f"TIME: Total time: {total_duration:.2f} seconds")
        print(f"OK: Tests passed: {tests_passed}/{total_tests}")
        print(f"RATE: Success rate: {(tests_passed/total_tests)*100:.1f}%")
        
        print("\nDETAILS: Test details:")
        for test_name, result in self.test_results.items():
            status_emoji = "OK" if result['success'] else "ERRO"
            duration = result.get('duration', 0)
            print(f"  {status_emoji} {test_name}: {result['status']} ({duration:.2f}s)")
        
        if tests_passed == total_tests:
            print("\nSUCCESS: ALL TESTS PASSED!")
            print("OK: Python SDK is working perfectly!")
            return True
        else:
            print(f"\nWARNING: {total_tests - tests_passed} test(s) failed")
            print("FIX: Check the errors above and fix the problems.")
            return False


def main():
    """Main function to run tests."""
    runner = TestRunner()
    success = runner.run_all_tests()
    
    # Exit code
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
