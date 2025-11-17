# Snapshot Cleanup 48h Proposal

## Why

Snapshots estão acumulando muito espaço em disco. O sistema atual mantém 7 dias de snapshots (168 snapshots com intervalo de 1 hora), o que pode ocupar muito espaço. Precisamos reduzir a retenção para 48 horas (2 dias) para economizar espaço em disco enquanto mantemos um histórico suficiente para recuperação.

## What Changes

- **MODIFIED**: Configuração padrão de retenção de snapshots de 7 dias para 2 dias (48 horas)
- **MODIFIED**: AutoSaveManager para usar retenção de 48 horas ao invés de 7 dias
- **MODIFIED**: SnapshotConfig default para retention_days: 2 e max_snapshots: 48 (24 snapshots/dia * 2 dias)
- **MODIFIED**: Limpeza automática de snapshots para manter apenas os últimos 48 horas

## Impact

- Affected specs: `storage`, `snapshot`
- Affected code: `src/storage/config.rs`, `src/storage/snapshot.rs`, `src/db/auto_save.rs`
- Breaking changes: Não (mudança de comportamento padrão, mas pode ser configurado)
- User benefit: Economia de espaço em disco, cleanup automático mais eficiente

