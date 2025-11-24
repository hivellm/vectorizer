# SDK Test Workflows

Este diretório contém workflows de CI/CD para testar todas as SDKs do Vectorizer antes de permitir publicação.

## Workflows Criados

### 1. TypeScript SDK Tests (`sdk-typescript-test.yml`)
- **Testa em**: Ubuntu, Windows, macOS
- **Versões Node.js**: 18.x, 20.x
- **Comandos**:
  - `npm ci` - Instala dependências
  - `npm run build` - Compila o SDK
  - `npm test` - Executa testes (skipa testes s2s)
- **Variáveis de ambiente**: `VITEST_SKIP_S2S=true`

### 2. JavaScript SDK Tests (`sdk-javascript-test.yml`)
- **Testa em**: Ubuntu, Windows, macOS
- **Versões Node.js**: 18.x, 20.x
- **Comandos**:
  - `npm ci` - Instala dependências
  - `npm run build` - Compila o SDK
  - `npm test` - Executa testes (skipa testes s2s)
- **Variáveis de ambiente**: `VITEST_SKIP_S2S=true`

### 3. Go SDK Tests (`sdk-go-test.yml`)
- **Testa em**: Ubuntu, Windows, macOS
- **Versões Go**: 1.21, 1.22
- **Comandos**:
  - `go mod download` - Baixa dependências
  - `go build` - Compila o SDK
  - `go test -v -short` - Executa testes unitários (skipa integração)
- **Variáveis de ambiente**: `SKIP_INTEGRATION_TESTS=true`

### 4. Python SDK Tests (`sdk-python-test.yml`)
- **Testa em**: Ubuntu, Windows, macOS
- **Versões Python**: 3.8, 3.9, 3.10, 3.11, 3.12
- **Comandos**:
  - `pip install -e ".[test,dev]"` - Instala dependências
  - `pytest tests/ -v -m "not integration"` - Executa testes unitários
- **Variáveis de ambiente**: `SKIP_INTEGRATION_TESTS=true`, `SKIP_S2S_TESTS=true`

### 5. Rust SDK Tests (`sdk-rust-test.yml`)
- **Testa em**: Ubuntu, Windows, macOS
- **Comandos**:
  - `cargo build --tests` - Compila testes
  - `cargo test --lib` - Executa testes unitários
  - `cargo clippy` - Verifica código
  - `cargo fmt --check` - Verifica formatação
- **Variáveis de ambiente**: `SKIP_INTEGRATION_TESTS=true`, `SKIP_S2S_TESTS=true`

### 6. C# SDK Tests (`sdk-csharp-test.yml`)
- **Testa em**: Ubuntu, Windows, macOS
- **Versão .NET**: 8.0.x
- **Comandos**:
  - `dotnet restore` - Restaura dependências
  - `dotnet build` - Compila o SDK
  - `dotnet test Vectorizer.Tests/` - Executa testes
- **Variáveis de ambiente**: `SKIP_INTEGRATION_TESTS=true`, `SKIP_S2S_TESTS=true`

### 7. All SDKs Tests Summary (`sdk-all-tests.yml`)
- **Tipo**: Workflow de resumo
- **Trigger**: Executado quando qualquer workflow de SDK completa
- **Função**: Agrega resultados de todos os workflows de SDK

## Como Funciona

### Triggers
Cada workflow é executado quando:
- Há push para branches `master`, `main`, ou `develop`
- Há pull request para qualquer branch
- Arquivos relacionados à SDK específica são modificados

### Testes S2S (Server-to-Server)
Testes que requerem servidor ativo são automaticamente skipados usando:
- Variáveis de ambiente (`SKIP_INTEGRATION_TESTS`, `SKIP_S2S_TESTS`)
- Filtros de teste (`-m "not integration"`, `-short`, etc.)
- Testes marcados com `it.skip()` ou `describe.skip()`

### Garantias
- ✅ Nenhuma SDK será publicada com testes falhando
- ✅ Testes são executados em múltiplos sistemas operacionais
- ✅ Múltiplas versões de runtime são testadas
- ✅ Build é verificado antes de executar testes
- ✅ Linting e formatação são verificados

## Executando Localmente

Para testar localmente antes de fazer push:

### TypeScript/JavaScript
```bash
cd sdks/typescript  # ou sdks/javascript
npm ci
npm run build
npm test
```

### Go
```bash
cd sdks/go
go mod download
go test -v -short ./...
```

### Python
```bash
cd sdks/python
pip install -e ".[test,dev]"
pytest tests/ -v -m "not integration"
```

### Rust
```bash
cd sdks/rust
cargo build --tests
cargo test --lib
cargo clippy -- -D warnings
cargo fmt --check
```

### C#
```bash
cd sdks/csharp
dotnet restore
dotnet build
dotnet test Vectorizer.Tests/
```

## Troubleshooting

### Testes falhando no CI mas passando localmente
1. Verifique se está usando as mesmas versões de runtime
2. Verifique se variáveis de ambiente estão configuradas
3. Verifique se dependências estão atualizadas

### Build falhando
1. Verifique se todas as dependências estão no `package.json`/`go.mod`/`requirements.txt`
2. Verifique se não há arquivos temporários sendo commitados
3. Verifique se o cache está funcionando corretamente

### Testes s2s sendo executados
1. Verifique se variáveis de ambiente estão sendo passadas
2. Verifique se testes estão marcados com `skip` ou filtros corretos
3. Verifique se o framework de teste está respeitando os filtros

## Referências

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Protobuf Workflows](https://github.com/protocolbuffers/protobuf/tree/main/.github/workflows) - Inspiração para estrutura

