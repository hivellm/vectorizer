# Sistema de Snapshot Completo de Memória

## 📋 Visão Geral

Este sistema foi desenvolvido para resolver a discrepância entre dados estimados e reais de memória no Vectorizer. Ele fornece snapshots precisos e detalhados do uso de memória, identificando exatamente onde estão as diferenças e fornecendo insights para otimização.

## 🎯 Problema Resolvido

**Antes**: Dados estimados não correspondiam ao uso real da API
**Agora**: Snapshot completo com dados reais do sistema operacional

## 🚀 Funcionalidades

### 1. **API Endpoints**
- `GET /api/v1/memory-snapshot` - Gera snapshot completo
- `POST /api/v1/memory-snapshot/export` - Exporta snapshot para arquivo

### 2. **Ferramentas CLI**
- **Python CLI**: Análise detalhada e comparação
- **PowerShell Script**: Monitoramento contínuo
- **Analisador de Discrepâncias**: Identificação de problemas específicos

### 3. **Dados Capturados**
- **Sistema**: Memória total, disponível, swap
- **Processo**: Memória física, virtual, heap, stack
- **Coleções**: Uso real vs estimado por coleção
- **Discrepâncias**: Análise detalhada de diferenças
- **Recomendações**: Sugestões específicas de otimização

## 📊 Estrutura do Snapshot

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "system_info": {
    "total_memory_mb": 16384.0,
    "available_memory_mb": 8192.0,
    "used_memory_mb": 8192.0,
    "memory_usage_percent": 50.0,
    "swap_total_mb": 4096.0,
    "swap_used_mb": 0.0
  },
  "process_info": {
    "physical_memory_mb": 2048.0,
    "virtual_memory_mb": 4096.0,
    "heap_size_mb": 1638.4,
    "stack_size_mb": 102.4,
    "shared_memory_mb": 307.2
  },
  "collections_info": {
    "total_collections": 25,
    "total_vectors": 1500000,
    "estimated_memory_mb": 1200.0,
    "actual_memory_mb": 800.0,
    "collections_detail": [...]
  },
  "discrepancy_analysis": {
    "estimated_vs_real_diff_mb": 400.0,
    "estimated_vs_real_diff_percent": 33.3,
    "unaccounted_memory_mb": 1248.0,
    "overhead_percentage": 156.0,
    "quantization_impact_mb": 300.0,
    "data_structure_overhead_mb": 948.0
  },
  "recommendations": [
    "✅ DashMap → HashMap+Mutex migration completed (740MB saved)",
    "💡 QUANTIZATION: Only 12/25 collections use quantization",
    "🔧 DATA STRUCTURES: 948.0MB overhead from data structures"
  ]
}
```

## 🛠️ Como Usar

### 1. **Via API**

```bash
# Gerar snapshot
curl -X GET http://localhost:8080/api/v1/memory-snapshot

# Exportar para arquivo
curl -X POST http://localhost:8080/api/v1/memory-snapshot/export \
  -H "Content-Type: application/json" \
  -d '{"file_path": "memory_report.json"}'
```

### 2. **Via Python CLI**

```bash
# Gerar snapshot e analisar
python scripts/memory_snapshot_cli.py --action snapshot --analyze

# Gerar snapshot e salvar
python scripts/memory_snapshot_cli.py --action snapshot --output memory_report.json

# Analisar snapshot existente
python scripts/memory_snapshot_cli.py --action analyze --input memory_report.json

# Comparar dois snapshots
python scripts/memory_snapshot_cli.py --action compare --input1 old.json --input2 new.json

# Monitoramento contínuo
python scripts/memory_snapshot_cli.py --action monitor --interval 30 --output memory_monitor.json
```

### 3. **Via PowerShell**

```powershell
# Monitoramento básico
.\scripts\memory_monitor.ps1

# Monitoramento com análise detalhada
.\scripts\memory_monitor.ps1 -Analyze -IntervalSeconds 60

# Monitoramento com export via API
.\scripts\memory_monitor.ps1 -ExportToFile -MaxSnapshots 50
```

### 4. **Análise de Discrepâncias**

```bash
# Analisar snapshot específico
python scripts/memory_discrepancy_analyzer.py --snapshot memory_report.json

# Comparar snapshots
python scripts/memory_discrepancy_analyzer.py --compare old.json new.json

# Monitoramento contínuo com análise
python scripts/memory_discrepancy_analyzer.py --monitor --interval 60
```

## 📈 Interpretando os Resultados

### **Score de Saúde da Memória (0-100)**
- **90-100**: Excelente - Sistema bem otimizado
- **70-89**: Bom - Algumas otimizações possíveis
- **50-69**: Regular - Necessita atenção
- **0-49**: Crítico - Ação imediata necessária

### **Overhead Percentual**
- **< 20%**: Baixo overhead - Bom
- **20-50%**: Overhead moderado - Aceitável
- **50-100%**: Overhead alto - Necessita otimização
- **> 100%**: Overhead excessivo - Crítico

### **Memória Não Contabilizada**
- **< 100 MB**: Normal
- **100-500 MB**: Investigar estruturas de dados
- **> 500 MB**: Possível vazamento de memória

## 🔍 Tipos de Análise

### 1. **Análise de Discrepância**
- Compara dados estimados vs reais
- Identifica overhead de estruturas de dados
- Calcula impacto da quantização

### 2. **Análise de Tendências**
- Monitora mudanças ao longo do tempo
- Detecta degradação de performance
- Identifica padrões de crescimento

### 3. **Análise de Otimização**
- Identifica oportunidades de quantização
- Sugere melhorias de estruturas de dados
- Calcula economia potencial

## 🚨 Alertas e Recomendações

### **Problemas Críticos**
- Overhead > 100%
- Memória não contabilizada > 500 MB
- Score de saúde < 50

### **Avisos**
- Overhead > 50%
- Quantização < 50% das coleções
- Coleções grandes sem otimização

### **Oportunidades**
- Ativar quantização em mais coleções
- Implementar lazy loading
- Revisar estruturas de dados

## 📁 Estrutura de Arquivos

```
vectorizer/
├── src/api/
│   ├── memory_snapshot.rs      # Lógica principal do snapshot
│   ├── memory_handlers.rs      # Endpoints da API
│   └── routes.rs              # Rotas adicionadas
├── scripts/
│   ├── memory_snapshot_cli.py  # CLI Python
│   ├── memory_monitor.ps1      # Monitor PowerShell
│   └── memory_discrepancy_analyzer.py  # Analisador
└── docs/
    └── MEMORY_SNAPSHOT_SYSTEM.md  # Esta documentação
```

## 🔧 Configuração

### **Variáveis de Ambiente**
```bash
# URL da API (padrão: http://localhost:8080/api/v1)
export VECTORIZER_API_URL="http://localhost:8080/api/v1"

# Intervalo de monitoramento (padrão: 30s)
export MEMORY_MONITOR_INTERVAL=30

# Diretório de saída (padrão: memory_snapshots)
export MEMORY_OUTPUT_DIR="memory_snapshots"
```

### **Dependências**
```bash
# Python
pip install requests

# PowerShell (Windows)
# Nenhuma dependência adicional necessária
```

## 📊 Exemplos de Uso

### **Cenário 1: Investigação de Alto Uso de Memória**

```bash
# 1. Gerar snapshot inicial
python scripts/memory_snapshot_cli.py --action snapshot --output baseline.json

# 2. Analisar discrepâncias
python scripts/memory_discrepancy_analyzer.py --snapshot baseline.json

# 3. Monitorar por 10 minutos
python scripts/memory_snapshot_cli.py --action monitor --interval 60 --output monitor.json

# 4. Comparar baseline vs final
python scripts/memory_discrepancy_analyzer.py --compare baseline.json monitor_20240115_103000_010.json
```

### **Cenário 2: Otimização Contínua**

```powershell
# Monitoramento contínuo com análise
.\scripts\memory_monitor.ps1 -Analyze -IntervalSeconds 120 -MaxSnapshots 20

# Análise de tendências
python scripts/memory_discrepancy_analyzer.py --monitor --interval 300
```

### **Cenário 3: Validação de Otimizações**

```bash
# Antes da otimização
python scripts/memory_snapshot_cli.py --action snapshot --output before.json

# Aplicar otimizações...

# Depois da otimização
python scripts/memory_snapshot_cli.py --action snapshot --output after.json

# Comparar resultados
python scripts/memory_discrepancy_analyzer.py --compare before.json after.json
```

## 🎯 Benefícios

1. **Visibilidade Real**: Dados precisos do sistema operacional
2. **Identificação de Problemas**: Detecta vazamentos e overhead
3. **Otimização Guiada**: Recomendações específicas e mensuráveis
4. **Monitoramento Contínuo**: Acompanhamento de tendências
5. **Comparação Histórica**: Análise de mudanças ao longo do tempo

## 🔮 Próximos Passos

1. **Dashboard Web**: Interface visual para análise
2. **Alertas Automáticos**: Notificações por email/Slack
3. **Integração com Métricas**: Prometheus/Grafana
4. **Análise Preditiva**: ML para prever problemas
5. **Otimização Automática**: Ajustes automáticos baseados em análise

## 📞 Suporte

Para dúvidas ou problemas:
1. Verifique os logs do Vectorizer
2. Confirme se a API está acessível
3. Valide as permissões de arquivo
4. Consulte os exemplos de uso

---

**Sistema desenvolvido para resolver discrepâncias de memória e fornecer insights precisos para otimização do Vectorizer.**
