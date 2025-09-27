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
        """Executa testes básicos."""
        print("🧪 Executando Testes Básicos")
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
            print(f"❌ Erro ao executar testes básicos: {e}")
            
            self.test_results['basic'] = {
                'success': False,
                'duration': duration,
                'status': 'ERROR',
                'error': str(e)
            }
            
            return False
    
    def run_comprehensive_tests(self):
        """Executa testes abrangentes."""
        print("\n🧪 Executando Testes Abrangentes")
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
            print(f"❌ Erro ao executar testes abrangentes: {e}")
            
            self.test_results['comprehensive'] = {
                'success': False,
                'duration': duration,
                'status': 'ERROR',
                'error': str(e)
            }
            
            return False
    
    def run_import_tests(self):
        """Testa imports de todos os módulos."""
        print("\n🧪 Testando Imports")
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
                print(f"✅ {module_name} - OK")
                import_results[module_name] = True
            except Exception as e:
                print(f"❌ {module_name} - ERRO: {e}")
                import_results[module_name] = False
        
        all_imports_ok = all(import_results.values())
        
        self.test_results['imports'] = {
            'success': all_imports_ok,
            'results': import_results,
            'status': 'PASSED' if all_imports_ok else 'FAILED'
        }
        
        return all_imports_ok
    
    def run_syntax_tests(self):
        """Testa sintaxe de todos os arquivos Python."""
        print("\n🧪 Testando Sintaxe")
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
                    print(f"✅ {filename} - Sintaxe OK")
                    syntax_results[filename] = True
                    
                except SyntaxError as e:
                    print(f"❌ {filename} - Erro de sintaxe: {e}")
                    syntax_results[filename] = False
                except Exception as e:
                    print(f"❌ {filename} - Erro: {e}")
                    syntax_results[filename] = False
            else:
                print(f"⚠️  {filename} - Arquivo não encontrado")
                syntax_results[filename] = False
        
        all_syntax_ok = all(syntax_results.values())
        
        self.test_results['syntax'] = {
            'success': all_syntax_ok,
            'results': syntax_results,
            'status': 'PASSED' if all_syntax_ok else 'FAILED'
        }
        
        return all_syntax_ok
    
    def run_all_tests(self):
        """Executa todos os testes."""
        print("🚀 EXECUTANDO TODOS OS TESTES DO SDK PYTHON")
        print("=" * 60)
        
        start_time = time.time()
        
        # Executar testes em ordem
        tests_passed = 0
        total_tests = 4
        
        # 1. Testes de sintaxe
        if self.run_syntax_tests():
            tests_passed += 1
        
        # 2. Testes de imports
        if self.run_import_tests():
            tests_passed += 1
        
        # 3. Testes básicos
        if self.run_basic_tests():
            tests_passed += 1
        
        # 4. Testes abrangentes (opcional)
        try:
            if self.run_comprehensive_tests():
                tests_passed += 1
        except ImportError:
            print("\n⚠️  Testes abrangentes não disponíveis (dependências ausentes)")
            total_tests = 3
        
        total_duration = time.time() - start_time
        
        # Relatório final
        print("\n" + "=" * 60)
        print("📊 RELATÓRIO FINAL DOS TESTES")
        print("=" * 60)
        
        print(f"⏱️  Tempo total: {total_duration:.2f} segundos")
        print(f"✅ Testes passaram: {tests_passed}/{total_tests}")
        print(f"📈 Taxa de sucesso: {(tests_passed/total_tests)*100:.1f}%")
        
        print("\n📋 Detalhes dos testes:")
        for test_name, result in self.test_results.items():
            status_emoji = "✅" if result['success'] else "❌"
            duration = result.get('duration', 0)
            print(f"  {status_emoji} {test_name}: {result['status']} ({duration:.2f}s)")
        
        if tests_passed == total_tests:
            print("\n🎉 TODOS OS TESTES PASSARAM!")
            print("✅ SDK Python está funcionando perfeitamente!")
            return True
        else:
            print(f"\n⚠️  {total_tests - tests_passed} teste(s) falharam")
            print("🔧 Verifique os erros acima e corrija os problemas.")
            return False


def main():
    """Função principal para executar testes."""
    runner = TestRunner()
    success = runner.run_all_tests()
    
    # Código de saída
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
