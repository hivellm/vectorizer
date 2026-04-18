# Proposal: phase1_remove-password-stdout-logging

## Why

`src/server/auth_handlers.rs:378-405` uses `println!()` to dump the auto-generated root admin credentials (username + password in cleartext) to stdout on first boot. This output lands in:

- container logs (captured by Docker, Kubernetes, Podman, etc.)
- CI/CD runners
- systemd journal
- developer terminal scrollback
- any log shipper in front of the container (Loki, Fluentd, CloudWatch...)

Anyone with access to any of those surfaces inherits root on the Vectorizer instance. Even after password rotation, the original value is preserved in log archives. This is the second most severe finding in the security audit.

## What Changes

1. **Stop logging the password**. The password must never leave process memory in cleartext.
2. **Write a one-shot credential file** at `data/.root_credentials` (mode 0600) that contains the generated password. The server prints only: `"Root credentials written to <path>. Read once and delete."`
3. **Or use a first-login token**: generate a short-lived setup token, print its URL (not the password), and make the operator set their own password via `/setup` on first visit.
4. Audit every other `println!`/`tracing::*` call site for secrets: JWT tokens in debug logs, API keys in error messages, bcrypt hashes in traces. Redact using a `Secret<T>` newtype wrapper.
5. Add a lint rule (custom clippy or grep CI gate) that flags `println!`/`info!`/`debug!` inside `src/auth/` and `src/server/auth_handlers.rs`.

## Impact

- Affected specs: auth module spec; logging spec in `/.rulebook/specs/`
- Affected code: `src/server/auth_handlers.rs`, `src/auth/mod.rs`, `src/auth/persistence.rs`, `src/server/mod.rs` (startup banner)
- Breaking change: operator UX changes (no more password in logs) — documented in release notes
- User benefit: closes log-exfiltration attack vector; compliant with common security baselines (CIS, SOC2 logging controls).
