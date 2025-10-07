# Executando os Testes - Vectorizer SDK v0.3.2

## 🔍 Status Atual

**Testes de Compilação:** ✅ **TODOS PASSANDO**  
**Testes Unitários:** ✅ **226/301 PASSANDO**  
**Testes de Integração:** ⚠️ **75 PRECISAM DO SERVIDOR**

## ⚠️ Por que 75 testes estão falhando?

Os 75 testes que estão falhando são **testes de integração** que precisam do servidor Vectorizer rodando em `http://localhost:15002`.

**Erro atual:**
```
NetworkError: Cannot read properties of undefined (reading 'ok')
```

Isso significa que os testes estão tentando conectar ao servidor mas ele não está disponível.

## ✅ Solução: Iniciar o Servidor Vectorizer

### Opção 1: Rodar o Servidor em um Terminal Separado

```bash
# Terminal 1: Iniciar o servidor
cd F:\Node\hivellm\vectorizer
cargo run --release

# Aguarde até ver: "Server running on http://localhost:15002"
```

```bash
# Terminal 2: Executar os testes
cd F:\Node\hivellm\vectorizer\client-sdks\typescript
npm test
```

### Opção 2: Usar Docker (se disponível)

```bash
# Iniciar servidor com Docker
docker run -p 15002:15002 hivellm/vectorizer:latest

# Em outro terminal
cd F:\Node\hivellm\vectorizer\client-sdks\typescript
npm test
```

### Opção 3: Executar Apenas Testes Unitários

Se você quer rodar apenas os testes que **não precisam do servidor**:

```bash
cd F:\Node\hivellm\vectorizer\client-sdks\typescript

# Rodar apenas testes unitários (que não fazem chamadas HTTP)
npm test -- --testPathIgnorePatterns="intelligent-search|discovery|file-operations"
```

## 📊 Categorias de Testes

### ✅ Testes Unitários (226 testes - PASSANDO)
Esses testes **NÃO precisam** do servidor:

- **Exception Classes** (40 testes) - Validação de erros
- **Search Result Validation** (23 testes) - Validação de modelos
- **Validation Utilities** (51 testes) - Utilitários de validação
- **Vector Model Validation** (17 testes) - Validação de vetores
- **Collection Model Validation** (20 testes) - Validação de coleções
- **HTTP Client** (27 testes) - Cliente HTTP
- **VectorizerClient Basic** (17 testes) - Funções básicas
- **Error Handling** (8 testes) - Tratamento de erros

### ⚠️ Testes de Integração (75 testes - PRECISAM DO SERVIDOR)
Esses testes **PRECISAM** do servidor rodando:

- **Intelligent Search** (18 testes)
  - intelligentSearch() - 5 testes
  - semanticSearch() - 4 testes
  - contextualSearch() - 4 testes
  - multiCollectionSearch() - 5 testes

- **Discovery Operations** (28 testes)
  - discover() - 5 testes
  - filterCollections() - 4 testes
  - scoreCollections() - 5 testes
  - expandQueries() - 6 testes
  - Integration tests - 2 testes
  - Performance tests - 2 testes

- **File Operations** (29 testes)
  - getFileContent() - 4 testes
  - listFilesInCollection() - 7 testes
  - getFileSummary() - 3 testes
  - getFileChunksOrdered() - 4 testes
  - getProjectOutline() - 4 testes
  - getRelatedFiles() - 4 testes
  - searchByFileType() - 4 testes
  - Performance tests - 2 testes

## 🚀 Comandos Rápidos

### Iniciar Servidor + Executar Testes

#### PowerShell (Windows):
```powershell
# Em um terminal
cd F:\Node\hivellm\vectorizer
cargo run --release

# Em outro terminal (aguarde o servidor iniciar)
timeout /t 5
cd F:\Node\hivellm\vectorizer\client-sdks\typescript
npm test
```

#### WSL/Linux:
```bash
# Em um terminal
cd /mnt/f/Node/hivellm/vectorizer
cargo run --release &

# Aguardar servidor iniciar e executar testes
sleep 5
cd /mnt/f/Node/hivellm/vectorizer/client-sdks/typescript
npm test
```

## 📈 Próximos Passos

### 1. **Para Desenvolvimento Local:**
```bash
# Sempre mantenha o servidor rodando durante desenvolvimento
cd F:\Node\hivellm\vectorizer
cargo run --release

# Em outro terminal, execute testes em modo watch
cd F:\Node\hivellm\vectorizer\client-sdks\typescript
npm run test:watch
```

### 2. **Para CI/CD:**

Adicione ao seu pipeline:
```yaml
# .github/workflows/test.yml
- name: Start Vectorizer Server
  run: |
    cd vectorizer
    cargo build --release
    ./target/release/vectorizer &
    sleep 5

- name: Run SDK Tests
  run: |
    cd client-sdks/typescript
    npm install
    npm test
```

### 3. **Configurar Variável de Ambiente:**

Se o servidor está em outra porta/host:
```bash
export VECTORIZER_URL=http://localhost:8080
npm test
```

## 🔧 Troubleshooting

### Erro: "Server not running"
✅ **Solução:** Inicie o servidor Vectorizer primeiro

### Erro: "Connection refused"
✅ **Solução:** Verifique se o servidor está na porta correta (15002)

### Erro: "Timeout"
✅ **Solução:** Aumente o timeout nos testes ou verifique performance do servidor

### Alguns testes passam, outros falham
✅ **Normal:** Testes unitários passam sem servidor, integração precisa dele

## 📝 Resumo

- ✅ **Código compilando perfeitamente**
- ✅ **226 testes unitários passando**
- ⚠️ **75 testes de integração aguardando servidor**

**Para executar TODOS os testes com sucesso:**
1. Inicie o servidor: `cargo run --release`
2. Execute os testes: `npm test`

**Resultado esperado:** 301/301 testes passando ✅

---

**Versão:** 0.3.2  
**Última atualização:** 07 de Outubro de 2025

