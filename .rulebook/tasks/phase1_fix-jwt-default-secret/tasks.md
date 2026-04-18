## 1. Implementation

- [ ] 1.1 Replace the hardcoded default in `src/auth/mod.rs:49` with `String::new()` (or remove `Default` impl entirely)
- [ ] 1.2 Add `AuthConfig::validate()` that rejects: empty secret, old default, length < 32, or low-entropy strings
- [ ] 1.3 Call `validate()` at server startup in `src/server/mod.rs` and return a fatal error with a remediation message if invalid
- [ ] 1.4 Add `src/auth/jwt_secret.rs` with `load_or_generate(path: &Path) -> Result<String>` using `rand::rngs::OsRng` + hex-encoding 64 bytes
- [ ] 1.5 Wire the generator into boot: if config secret is empty AND auto-gen is enabled, generate and persist to `data/jwt_secret.key` with 0600
- [ ] 1.6 Update `config.example.yml` to remove the placeholder secret; add comment `# REQUIRED - generate: openssl rand -hex 64`
- [ ] 1.7 Update `.env.example` and `.env.hub` accordingly
- [ ] 1.8 Update `config.production.yml` and `config.cluster.yml` to reference env var, never literal value

## 2. Tail (mandatory — enforced by rulebook v5.3.0)

- [ ] 2.1 Document the breaking change in `CHANGELOG.md` under `### Breaking` and add a migration note to `docs/security.md`
- [ ] 2.2 Write unit tests in `src/auth/mod.rs` covering: empty secret rejected, old default rejected, short secret rejected, valid 64-hex accepted; write integration test proving server refuses to boot with an invalid secret
- [ ] 2.3 Run `cargo test --all-features -- auth` and confirm all tests pass

## Mandatory tail (required by rulebook v5.3.0)

- [ ] Update or create documentation covering the implementation
- [ ] Write tests covering the new behavior
- [ ] Run tests and confirm they pass
