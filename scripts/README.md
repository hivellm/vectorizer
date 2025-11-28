# Vectorizer Installation Scripts

Scripts de instalação para Vectorizer que instalam diretamente do repositório Git.

## Linux/macOS

```bash
curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh | bash
```

Ou baixe e execute localmente:

```bash
curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh -o install.sh
chmod +x install.sh
./install.sh
```

## Windows PowerShell

```powershell
powershell -c "irm https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.ps1 | iex"
```

Ou baixe e execute localmente:

```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.ps1" -OutFile "install.ps1"
.\install.ps1
```

## Variáveis de Ambiente

### VECTORIZER_VERSION
Especifica a versão/tag a instalar. Padrão: `latest`

```bash
# Instalar versão específica
VECTORIZER_VERSION=1.3.0 curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh | bash
```

### VECTORIZER_INSTALL_DIR
Diretório onde o código será clonado. Padrão: `$HOME/.vectorizer` (Linux/macOS) ou `$USERPROFILE\.vectorizer` (Windows)

```bash
VECTORIZER_INSTALL_DIR=/opt/vectorizer curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh | bash
```

### VECTORIZER_BIN_DIR
Diretório onde o binário será instalado. Padrão: `/usr/local/bin` (Linux/macOS) ou `$USERPROFILE\.cargo\bin` (Windows)

```bash
VECTORIZER_BIN_DIR=/usr/bin curl -fsSL https://raw.githubusercontent.com/hivellm/vectorizer/main/scripts/install.sh | bash
```

## Requisitos

- **Git**: Necessário para clonar o repositório
- **Rust**: Será instalado automaticamente se não estiver presente
- **Cargo**: Incluído com Rust

## O que os scripts fazem

1. Verificam se Rust está instalado (instalam se necessário)
2. Clonam o repositório Git do Vectorizer
3. Compilam o projeto em modo release
4. Instalam o binário em um diretório no PATH
5. Verificam a instalação

## Desinstalação

Para desinstalar, simplesmente remova o binário:

```bash
# Linux/macOS
sudo rm /usr/local/bin/vectorizer

# Windows
Remove-Item "$env:USERPROFILE\.cargo\bin\vectorizer.exe"
```

Opcionalmente, você também pode remover o diretório de instalação:

```bash
# Linux/macOS
rm -rf ~/.vectorizer

# Windows
Remove-Item -Recurse -Force "$env:USERPROFILE\.vectorizer"
```
