# Proposal: phase5_consolidate-config-files

## Why

The project carries **five** top-level config files:

- `config.yml` — HiveHub integration (multi-tenant)
- `config.example.yml` — generic template
- `config.production.yml` — prod-optimized
- `config.cluster.yml` — cluster mode
- `config.hub.yml` — hub-specific

Spot-checking `logging.level` showed drift: `debug` / `debug` / `warn` / `warn` / `info` across the five files. Other keys agree, but the drift pattern means any config change is at risk of being applied to only some files.

Having five near-identical YAMLs invites:

- Security misconfigurations (one file has JWT secret placeholder, another doesn't).
- Inconsistent defaults across deployment targets.
- Confusion for new operators ("which one do I edit?").

## What Changes

1. Collapse to a single `config.example.yml` that documents every option with sane defaults.
2. Replace the four "mode-specific" configs with **override fragments** under `config/modes/`: `production.yml`, `cluster.yml`, `hub.yml`, `dev.yml` — each containing only the values that differ from the example.
3. Support config layering in `src/config/` (load base → apply mode override → apply env → apply CLI).
4. Document the layering in `docs/deployment/configuration.md`.
5. Add a test that loads each mode override and asserts the merged config validates.

## Impact

- Affected specs: configuration spec
- Affected code: `src/config/`, repo root (5 → 1 + override fragments)
- Breaking change: YES for operators — document migration. Provide a one-shot migration script.
- User benefit: single source of truth for defaults; impossible for mode configs to silently drift.
