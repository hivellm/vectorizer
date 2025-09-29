# 🚀 Vectorizer Release System

Este documento explica como o sistema de releases automáticos do Vectorizer funciona e como criar novos releases.

## 📋 **Visão Geral**

O sistema de releases do Vectorizer é totalmente automatizado e é executado quando uma tag de versão é criada no repositório. Ele gera builds para múltiplas plataformas e cria um release no GitHub automaticamente.

## 🔄 **Como Funciona**

### 1. **Trigger Automático**
- O release é executado quando uma tag no formato `v*.*.*` é criada
- Exemplos: `v0.22.0`, `v1.0.0`, `v2.5.3`
- **NÃO** executa para tags como `v1` ou `latest`

### 2. **Processo Automático**
1. **Validação da Tag**: Verifica se a tag segue o formato de versão semântica
2. **Build Multiplataforma**: Compila para 6 plataformas diferentes
3. **Criação de Arquivos**: Gera executáveis, scripts de instalação e documentação
4. **Release no GitHub**: Cria automaticamente o release com todos os arquivos

## 🖥️ **Plataformas Suportadas**

| Plataforma | Arquitetura | Formato | Script de Instalação |
|------------|-------------|---------|---------------------|
| **Linux** | x86_64 | `.tar.gz` | `install.sh` |
| **Linux** | aarch64 | `.tar.gz` | `install.sh` |
| **Windows** | x86_64 | `.zip` | `install.bat` |
| **Windows** | aarch64 | `.zip` | `install.bat` |
| **macOS** | x86_64 | `.tar.gz` | `install.sh` |
| **macOS** | aarch64 | `.tar.gz` | `install.sh` |

## 🛠️ **Como Criar um Release**

### **Opção 1: Script Automatizado (Recomendado)**

```bash
# Navegar para o diretório do projeto
cd /path/to/vectorizer

# Executar o script de release
./scripts/create_release.sh patch    # Para versão patch (0.22.0 -> 0.22.1)
./scripts/create_release.sh minor    # Para versão minor (0.22.0 -> 0.23.0)
./scripts/create_release.sh major    # Para versão major (0.22.0 -> 1.0.0)

# Ou especificar uma versão exata
./scripts/create_release.sh --version 1.5.0

# Preview do que será feito (sem executar)
./scripts/create_release.sh --dry-run patch
```

### **Opção 2: Manual**

```bash
# 1. Atualizar versão nos arquivos
# - Cargo.toml
# - client-sdks/*/package.json
# - client-sdks/*/setup.py
# - client-sdks/*/Cargo.toml

# 2. Commit das mudanças
git add .
git commit -m "chore: bump version to 0.23.0"

# 3. Criar tag
git tag -a "v0.23.0" -m "Release v0.23.0"

# 4. Push da tag
git push origin main
git push origin "v0.23.0"
```

## 📦 **Conteúdo do Release**

Cada release contém:

### **Executáveis**
- `vectorizer-cli` - Interface de linha de comando
- `vectorizer-server` - Servidor HTTP API (porta 15001)
- `vectorizer-mcp-server` - Servidor MCP Protocol (porta 15002)
- `vzr` - GRPC orchestrator e indexing engine (porta 15003)

### **Arquivos de Configuração**
- `config.yml` - Arquivo de configuração padrão
- `vectorize-workspace.yml` - Configuração do workspace
- `README.md` - Documentação
- `LICENSE` - Licença do projeto

### **Scripts de Instalação**
- **Linux/macOS**: `install.sh` - Instalação system-wide
- **Windows**: `install.bat` - Instalação system-wide

## 🔍 **Monitoramento**

### **GitHub Actions**
- Workflow: [Tag Release](https://github.com/hivellm/vectorizer/actions/workflows/tag-release.yml)
- Status: Monitore o progresso em tempo real

### **Release Page**
- URL: `https://github.com/hivellm/vectorizer/releases/tag/v{VERSION}`
- Downloads: Links diretos para todos os arquivos

## 🚨 **Troubleshooting**

### **Tag Inválida**
```bash
# ❌ Estas tags NÃO funcionam:
git tag v1          # Muito genérica
git tag latest      # Não é versão semântica
git tag v1.0        # Falta patch version

# ✅ Estas tags funcionam:
git tag v1.0.0      # Versão completa
git tag v0.22.1     # Versão patch
git tag v2.0.0-beta # Com prerelease
```

### **Falha no Build**
- Verifique se o código compila localmente: `cargo build --release`
- Verifique se todas as dependências estão atualizadas
- Verifique os logs do GitHub Actions para detalhes

### **Falha na Criação do Release**
- Verifique se a tag foi criada corretamente
- Verifique se o token `GITHUB_TOKEN` tem permissões adequadas
- Verifique se não existe um release com a mesma versão

## 📊 **Exemplo de Release**

### **Tag Criada**: `v0.22.0`

### **Arquivos Gerados**:
```
vectorizer-linux-x86_64.tar.gz     (Linux x86_64)
vectorizer-linux-aarch64.tar.gz    (Linux ARM64)
vectorizer-windows-x86_64.zip      (Windows x86_64)
vectorizer-windows-aarch64.zip     (Windows ARM64)
vectorizer-macos-x86_64.tar.gz     (macOS x86_64)
vectorizer-macos-aarch64.tar.gz    (macOS ARM64)
```

### **Release Notes**:
- Instruções de instalação para cada plataforma
- Links de download diretos
- Documentação de quick start
- Lista de features e changelog

## 🔧 **Configuração Avançada**

### **Workflows Disponíveis**
- `tag-release.yml` - Release completo com tag
- `build.yml` - Builds para branches (sem release)
- `ci.yml` - CI/CD completo com testes

### **Variáveis de Ambiente**
- `CARGO_TERM_COLOR=always` - Cores no output do Cargo
- `RUST_BACKTRACE=1` - Backtrace completo em erros

### **Cache**
- Cache automático do Cargo registry
- Cache de builds para acelerar compilação
- Retenção de artifacts por 30 dias

## 📝 **Boas Práticas**

1. **Semantic Versioning**: Use sempre o formato `vX.Y.Z`
2. **Changelog**: Atualize o CHANGELOG.md antes do release
3. **Testes**: Certifique-se de que todos os testes passam
4. **Documentação**: Atualize a documentação se necessário
5. **Preview**: Use `--dry-run` para verificar antes de executar

## 🤝 **Suporte**

Se você encontrar problemas com o sistema de releases:

1. Verifique os logs do GitHub Actions
2. Abra uma issue no repositório
3. Consulte a documentação do GitHub Actions
4. Verifique se a tag segue o formato correto

---

**🎉 Com este sistema, criar releases é simples e automático!**
