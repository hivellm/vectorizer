# 📖 **File Watcher User Guide**
## **Vectorizer - Real-time File Monitoring System**

**Versão**: 1.0  
**Data**: $(date)  
**Status**: ✅ **PRONTO PARA USO**

---

## 🎯 **O que é o File Watcher?**

O File Watcher é um sistema que monitora automaticamente mudanças em arquivos e atualiza o banco de dados vetorial do Vectorizer em tempo real. **Você não precisa mais reiniciar a aplicação** quando arquivos são modificados, adicionados ou removidos.

### **Problema que Resolve**
- ❌ **Antes**: Era necessário reiniciar o Vectorizer toda vez que arquivos mudavam
- ✅ **Agora**: Mudanças são detectadas e processadas automaticamente

---

## 🚀 **Como Usar**

### **Inicialização Automática**

O File Watcher é iniciado automaticamente quando você inicia o servidor Vectorizer:

```bash
# Iniciar o servidor (File Watcher inicia automaticamente)
cargo run --bin vectorizer -- --host 0.0.0.0 --port 8080
```

Você verá logs como:
```
🔍 Starting file watcher system...
✅ File watcher started successfully
```

### **Verificar se está Funcionando**

```bash
# Verificar status do servidor
curl http://localhost:8080/health

# Verificar logs do File Watcher
tail -f server.log | grep "file watcher\|FileWatcher"
```

---

## 📁 **Tipos de Arquivo Suportados**

O File Watcher monitora automaticamente estes tipos de arquivo:

| Extensão | Tipo | Descrição |
|----------|------|-----------|
| `.md` | Markdown | Documentação, READMEs |
| `.txt` | Texto | Arquivos de texto simples |
| `.rs` | Rust | Código fonte Rust |
| `.py` | Python | Código fonte Python |
| `.js` | JavaScript | Código fonte JavaScript |
| `.ts` | TypeScript | Código fonte TypeScript |
| `.json` | JSON | Arquivos de configuração |
| `.yaml/.yml` | YAML | Arquivos de configuração |

### **Arquivos Ignorados**

Estes arquivos são automaticamente ignorados:
- `**/target/**` - Diretórios de build do Rust
- `**/node_modules/**` - Dependências do Node.js
- `**/.git/**` - Arquivos do Git
- `**/*.tmp` - Arquivos temporários
- `**/*.log` - Arquivos de log
- `**/*.lock` - Arquivos de lock
- `**/Cargo.lock` - Lock file do Rust
- `**/.DS_Store` - Arquivos do macOS

---

## ⚙️ **Configuração**

### **Configuração Padrão**

O File Watcher usa uma configuração padrão que funciona para a maioria dos casos:

```yaml
# Configuração padrão (não precisa ser alterada)
watch_paths: null                    # Auto-descoberta de arquivos
debounce_delay_ms: 1000              # 1 segundo de delay
max_file_size: 10485760              # 10MB máximo
collection_name: "watched_files"     # Nome da coleção
recursive: true                      # Monitorar subdiretórios
enable_realtime_indexing: true       # Indexação em tempo real
```

### **Personalização (Avançado)**

Se precisar personalizar, você pode modificar a configuração no código:

```rust
// Exemplo de configuração personalizada
let mut config = FileWatcherConfig::default();
config.debounce_delay_ms = 2000;  // 2 segundos de delay
config.max_file_size = 20 * 1024 * 1024;  // 20MB máximo
config.collection_name = "meus_arquivos".to_string();

// Adicionar novos tipos de arquivo
config.include_patterns.push("*.cpp".to_string());
config.include_patterns.push("*.h".to_string());

// Excluir diretórios específicos
config.exclude_patterns.push("**/build/**".to_string());
```

---

## 🔍 **Como Funciona**

### **Fluxo de Processamento**

1. **Detecção**: O sistema detecta mudanças em arquivos
2. **Debouncing**: Aguarda 1 segundo para evitar processamento excessivo
3. **Filtragem**: Verifica se o arquivo deve ser processado
4. **Indexação**: Adiciona/atualiza o arquivo no banco vetorial
5. **Logging**: Registra a operação nos logs

### **Tipos de Eventos**

| Evento | Ação |
|--------|------|
| **Arquivo Criado** | Indexa automaticamente |
| **Arquivo Modificado** | Re-indexa automaticamente |
| **Arquivo Deletado** | Remove do índice automaticamente |
| **Arquivo Renomeado** | Remove o antigo e adiciona o novo |

---

## 📊 **Monitoramento**

### **Logs Importantes**

```bash
# Ver todos os logs do File Watcher
tail -f server.log | grep -E "(file watcher|FileWatcher|Indexed|Removed)"

# Ver apenas erros
tail -f server.log | grep -E "(ERROR|WARN).*file watcher"
```

### **Exemplos de Logs**

**Inicialização bem-sucedida:**
```
🔍 Starting file watcher system...
✅ File watcher started successfully
```

**Arquivo indexado:**
```
Indexed file: /path/to/document.md in collection: watched_files
```

**Arquivo removido:**
```
Removed file: /path/to/old_file.txt from collection: watched_files
```

**Arquivo ignorado:**
```
Skipping file (doesn't match patterns): /path/to/binary.exe
```

---

## 🛠️ **Solução de Problemas**

### **Problema: File Watcher não está funcionando**

**Sintomas:**
- Mudanças em arquivos não são detectadas
- Logs não mostram atividade do File Watcher

**Soluções:**
1. **Verificar se está iniciado:**
   ```bash
   grep "File watcher started successfully" server.log
   ```

2. **Verificar erros:**
   ```bash
   grep -i "error.*file watcher" server.log
   ```

3. **Reiniciar o servidor:**
   ```bash
   # Parar o servidor (Ctrl+C)
   # Iniciar novamente
   cargo run --bin vectorizer -- --host 0.0.0.0 --port 8080
   ```

### **Problema: Arquivos não são indexados**

**Sintomas:**
- Arquivos modificados não aparecem nas buscas
- Logs mostram "Skipping file"

**Soluções:**
1. **Verificar extensão do arquivo:**
   - Certifique-se de que a extensão está na lista suportada
   - Adicione novos tipos se necessário

2. **Verificar tamanho do arquivo:**
   - Arquivos maiores que 10MB são ignorados
   - Aumente o limite se necessário

3. **Verificar padrões de exclusão:**
   - Certifique-se de que o arquivo não está em um diretório excluído

### **Problema: Performance lenta**

**Sintomas:**
- Sistema lento ao processar muitos arquivos
- Alto uso de CPU/memória

**Soluções:**
1. **Aumentar delay de debouncing:**
   ```rust
   config.debounce_delay_ms = 2000;  // 2 segundos
   ```

2. **Reduzir tipos de arquivo monitorados:**
   ```rust
   config.include_patterns = vec!["*.md".to_string(), "*.txt".to_string()];
   ```

3. **Adicionar mais exclusões:**
   ```rust
   config.exclude_patterns.push("**/large_files/**".to_string());
   ```

---

## 📈 **Métricas e Performance**

### **Métricas Padrão**

| Métrica | Valor Padrão | Descrição |
|---------|--------------|-----------|
| **Debounce Delay** | 1000ms | Tempo de espera antes de processar |
| **Max File Size** | 10MB | Tamanho máximo de arquivo |
| **Max Concurrent Tasks** | 4 | Tarefas simultâneas |
| **Batch Size** | 100 | Tamanho do lote de processamento |

### **Monitoramento de Performance**

```bash
# Verificar uso de memória
ps aux | grep vectorizer

# Verificar atividade de arquivos
lsof | grep vectorizer

# Verificar logs de performance
grep "processing_time" server.log
```

---

## 🔧 **Comandos Úteis**

### **Verificar Status**
```bash
# Status do servidor
curl http://localhost:8080/health

# Listar coleções
curl http://localhost:8080/collections

# Buscar arquivos indexados
curl -X POST http://localhost:8080/search \
  -H "Content-Type: application/json" \
  -d '{"query": "meu arquivo", "collection": "watched_files"}'
```

### **Logs e Debug**
```bash
# Logs em tempo real
tail -f server.log

# Logs apenas do File Watcher
tail -f server.log | grep -E "(file watcher|FileWatcher)"

# Logs de erro
tail -f server.log | grep -i error

# Contar eventos processados
grep "Indexed file" server.log | wc -l
```

---

## 🎯 **Casos de Uso Comuns**

### **Desenvolvimento de Software**
- Monitora mudanças em código fonte
- Atualiza índice automaticamente
- Permite busca em tempo real no código

### **Documentação**
- Monitora arquivos Markdown
- Atualiza índice de documentação
- Facilita busca em documentação

### **Projetos de Dados**
- Monitora arquivos de configuração
- Atualiza índice de metadados
- Facilita busca em configurações

---

## ❓ **Perguntas Frequentes**

### **P: O File Watcher consome muitos recursos?**
**R:** Não. O sistema foi otimizado para baixo consumo:
- Debouncing evita processamento excessivo
- Filtragem de arquivos reduz carga
- Processamento assíncrono não bloqueia

### **P: Posso desabilitar o File Watcher?**
**R:** Sim, mas não é recomendado. O sistema foi projetado para ser leve e eficiente.

### **P: Como adicionar novos tipos de arquivo?**
**R:** Modifique a configuração para incluir novas extensões:
```rust
config.include_patterns.push("*.cpp".to_string());
```

### **P: O File Watcher funciona em todos os sistemas operacionais?**
**R:** Sim, funciona em Linux, macOS e Windows.

### **P: Posso monitorar arquivos em rede?**
**R:** Sim, desde que o sistema operacional suporte notificações de arquivo em rede.

---

## 🎉 **Conclusão**

O File Watcher torna o Vectorizer muito mais conveniente de usar:

- ✅ **Sem reinicializações** - Mudanças são detectadas automaticamente
- ✅ **Tempo real** - Índice sempre atualizado
- ✅ **Configurável** - Adapta-se às suas necessidades
- ✅ **Eficiente** - Baixo consumo de recursos
- ✅ **Confiável** - Error handling robusto

**Agora você pode focar no seu trabalho sem se preocupar em reiniciar o Vectorizer!**

---

**Guia do usuário gerado em**: $(date)  
**Versão**: 1.0  
**Status**: ✅ **PRONTO PARA USO**
