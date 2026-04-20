//! Layered config loader: **base → mode → env → CLI**.
//!
//! See `.rulebook/tasks/2026-04-19-phase5_consolidate-config-files/design.md`
//! for the per-section default decisions and the rationale behind the
//! merge semantics.
//!
//! # Use
//!
//! ```no_run
//! use std::path::Path;
//! use vectorizer::config::layered::{load_layered, LayeredOptions};
//!
//! let cfg = load_layered(
//!     Path::new("config/config.yml"),
//!     LayeredOptions {
//!         mode: std::env::var("VECTORIZER_MODE").ok().as_deref().map(str::to_owned),
//!         modes_dir: None, // defaults to <base_dir>/modes/
//!     },
//! )?;
//! # Ok::<(), vectorizer::config::layered::ConfigError>(())
//! ```
//!
//! # Merge semantics
//!
//! Mode overrides are applied as a **deep YAML merge** on top of the
//! parsed base document:
//!
//! - **Scalar** values in the override replace the base scalar.
//! - **Map** values are merged key-by-key, recursively.
//! - **Array** values fully replace the base array. There is no
//!   element-level merge — the natural shape (e.g. replacing the whole
//!   `cluster.servers: [...]` list) is what operators expect.
//! - **Null** in the override clears the base value (rarely needed, but
//!   the only way to "unset" a field that has a non-null base default).
//!
//! After the merge, the resulting [`serde_yaml::Value`] is deserialized
//! into the strict [`crate::config::VectorizerConfig`] struct, which
//! carries the actual validation (unknown keys are tolerated at the
//! merge layer because YAML can't tell them apart from typos; serde's
//! `#[serde(default)]` + per-section validation is the source of truth
//! for "is this a real config?").

use std::path::{Path, PathBuf};

use serde_yaml::Value;
use thiserror::Error;
use tracing::{debug, info, warn};

use super::VectorizerConfig;

/// Options for [`load_layered`].
#[derive(Debug, Clone, Default)]
pub struct LayeredOptions {
    /// The mode name to apply on top of the base. `None` means "no
    /// mode override; load the base alone".
    pub mode: Option<String>,
    /// Override the lookup directory for mode files. Defaults to
    /// `<base_dir>/modes/` — i.e. `config/modes/` when the base lives
    /// at `config/config.yml` per the canonical layout. Tests
    /// typically pass an explicit path; production callers leave this
    /// at `None`.
    pub modes_dir: Option<PathBuf>,
}

/// Errors the layered loader can produce.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The base config file (typically `config.yml`) is missing.
    #[error("base config file not found at '{0}'")]
    BaseNotFound(PathBuf),

    /// A mode was requested but no override file exists for it.
    #[error("config mode '{mode}' not found at '{path}'")]
    ModeNotFound {
        /// Requested mode name.
        mode: String,
        /// Path the loader looked at.
        path: PathBuf,
    },

    /// I/O failure reading one of the layer files.
    #[error("failed to read config file '{path}': {source}")]
    Io {
        /// Path the loader was reading.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// The base or override file isn't valid YAML.
    #[error("failed to parse YAML from '{path}': {source}")]
    Parse {
        /// Path the loader was parsing.
        path: PathBuf,
        /// Underlying serde-yaml error.
        source: serde_yaml::Error,
    },

    /// The merged document doesn't match the [`VectorizerConfig`]
    /// schema. Most often a section type mismatch in an override.
    #[error("merged config failed schema validation: {0}")]
    Schema(String),
}

/// Load the base config, optionally apply a mode override, and return
/// the strict [`VectorizerConfig`].
pub fn load_layered(
    base_path: &Path,
    opts: LayeredOptions,
) -> Result<VectorizerConfig, ConfigError> {
    if !base_path.exists() {
        return Err(ConfigError::BaseNotFound(base_path.to_path_buf()));
    }

    let base_yaml = read_yaml(base_path)?;
    debug!(path = %base_path.display(), "loaded base config");

    let merged = match opts.mode.as_deref() {
        None => {
            info!("no mode override requested; using base config alone");
            base_yaml
        }
        Some(mode) => {
            // Mode overrides live in `<base_dir>/modes/`. Under the
            // canonical phase4_consolidate-repo-layout layout that
            // resolves to `config/modes/` (base_path =
            // `config/config.yml`); under the legacy
            // `./config.yml`-at-root layout it resolves to `./modes/`,
            // which the deprecation shim warns about at boot.
            let modes_dir = opts.modes_dir.unwrap_or_else(|| {
                base_path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join("modes")
            });
            let override_path = modes_dir.join(format!("{mode}.yml"));
            if !override_path.exists() {
                return Err(ConfigError::ModeNotFound {
                    mode: mode.to_owned(),
                    path: override_path,
                });
            }
            let override_yaml = read_yaml(&override_path)?;
            info!(
                path = %override_path.display(),
                mode = %mode,
                "applying mode override on top of base"
            );
            merge_yaml(base_yaml, override_yaml)
        }
    };

    serde_yaml::from_value(merged).map_err(|e| ConfigError::Schema(e.to_string()))
}

/// Deep merge `override_value` into `base`. See module docs for
/// semantics. Public for testing — production callers go through
/// [`load_layered`].
pub fn merge_yaml(base: Value, override_value: Value) -> Value {
    match (base, override_value) {
        // Map ⊕ Map: recursive merge.
        (Value::Mapping(mut base_map), Value::Mapping(override_map)) => {
            for (k, v) in override_map {
                let merged = match base_map.remove(&k) {
                    Some(existing) => merge_yaml(existing, v),
                    None => v,
                };
                base_map.insert(k, merged);
            }
            Value::Mapping(base_map)
        }
        // Anything ⊕ Null in override means "unset". Yields Null.
        (_, Value::Null) => Value::Null,
        // Anything else: override wins. (Scalars replace scalars,
        // arrays fully replace arrays, type changes replace.)
        (_, override_value) => override_value,
    }
}

fn read_yaml(path: &Path) -> Result<Value, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_yaml::from_str(&content).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

/// Parse the value of `VECTORIZER_MODE` env into an `Option<String>`
/// suitable for [`LayeredOptions::mode`]. Empty string and unset are
/// both treated as "no mode override".
pub fn mode_from_env() -> Option<String> {
    match std::env::var("VECTORIZER_MODE") {
        Ok(s) if !s.trim().is_empty() => Some(s.trim().to_owned()),
        _ => {
            // Common operator slip: typo'd env var name. Surface it
            // at debug level so it shows up in diagnostic captures
            // without polluting normal logs.
            if std::env::vars().any(|(k, _)| {
                let lower = k.to_lowercase();
                lower.contains("vectorizer") && lower.contains("mode") && k != "VECTORIZER_MODE"
            }) {
                warn!(
                    "found a `VECTORIZER*MODE*` env var that is NOT exactly \
                     `VECTORIZER_MODE` — mode override may not be applied as \
                     expected"
                );
            }
            None
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn yaml(s: &str) -> Value {
        serde_yaml::from_str(s).expect("test fixture must be valid YAML")
    }

    /// Minimum-shape base YAML that satisfies `VectorizerConfig`'s
    /// strict deserializer (the three sections without
    /// `#[serde(default)]`: `server`, `file_watcher`, `logging`).
    /// Every other section relies on its struct-level Default impl.
    const MINIMUM_BASE: &str = r#"
server:
  host: 127.0.0.1
  port: 15002
  mcp_port: 15002
file_watcher:
  enabled: false
  collection_name: workspace-files
logging:
  level: info
  log_requests: true
  log_responses: false
  log_errors: true
"#;

    #[test]
    fn merge_scalar_replaces_scalar() {
        let base = yaml("a: 1");
        let over = yaml("a: 2");
        let merged = merge_yaml(base, over);
        assert_eq!(merged, yaml("a: 2"));
    }

    #[test]
    fn merge_map_recurses_per_key() {
        let base = yaml(
            r#"
            server:
              host: 127.0.0.1
              port: 15002
              mcp_port: 15002
            "#,
        );
        let over = yaml(
            r#"
            server:
              host: 0.0.0.0
            "#,
        );
        let merged = merge_yaml(base, over);
        let expected = yaml(
            r#"
            server:
              host: 0.0.0.0
              port: 15002
              mcp_port: 15002
            "#,
        );
        assert_eq!(merged, expected);
    }

    #[test]
    fn merge_array_replaces_whole_array() {
        // Per design.md, array merge is "override fully replaces".
        // The natural shape for cluster.servers is "give me your full
        // list, not a per-element merge".
        let base = yaml(
            r#"
            cluster:
              servers:
                - {id: a}
                - {id: b}
            "#,
        );
        let over = yaml(
            r#"
            cluster:
              servers:
                - {id: c}
            "#,
        );
        let merged = merge_yaml(base, over);
        let expected = yaml(
            r#"
            cluster:
              servers:
                - {id: c}
            "#,
        );
        assert_eq!(merged, expected);
    }

    #[test]
    fn merge_null_in_override_unsets_base_value() {
        let base = yaml(
            r#"
            auth:
              jwt_secret: from-base
            "#,
        );
        let over = yaml(
            r#"
            auth:
              jwt_secret: null
            "#,
        );
        let merged = merge_yaml(base, over);
        let expected = yaml(
            r#"
            auth:
              jwt_secret: null
            "#,
        );
        assert_eq!(merged, expected);
    }

    #[test]
    fn merge_override_only_keys_are_added_to_base() {
        let base = yaml(
            r#"
            storage:
              wal:
                enabled: true
            "#,
        );
        let over = yaml(
            r#"
            storage:
              compression:
                enabled: true
                format: zstd
            "#,
        );
        let merged = merge_yaml(base, over);
        let expected = yaml(
            r#"
            storage:
              wal:
                enabled: true
              compression:
                enabled: true
                format: zstd
            "#,
        );
        assert_eq!(merged, expected);
    }

    #[test]
    fn merge_type_change_lets_override_win() {
        // Operator typo: the base has a map, the override has a scalar
        // for the same key. The override wins (yields the scalar);
        // serde-side schema validation will then reject if the resulting
        // shape doesn't match the typed config.
        let base = yaml("section: {a: 1}");
        let over = yaml("section: 42");
        let merged = merge_yaml(base, over);
        assert_eq!(merged, yaml("section: 42"));
    }

    #[test]
    fn missing_base_returns_typed_error() {
        let opts = LayeredOptions::default();
        let err = load_layered(Path::new("does-not-exist.yml"), opts).unwrap_err();
        match err {
            ConfigError::BaseNotFound(p) => {
                assert_eq!(p, Path::new("does-not-exist.yml"));
            }
            other => panic!("expected BaseNotFound, got {other:?}"),
        }
    }

    #[test]
    fn missing_mode_override_returns_typed_error() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let base_path = tmp.path().join("config.yml");
        std::fs::write(&base_path, MINIMUM_BASE).expect("write base");

        let opts = LayeredOptions {
            mode: Some("nonexistent".to_owned()),
            modes_dir: Some(tmp.path().join("config").join("modes")),
        };
        let err = load_layered(&base_path, opts).unwrap_err();
        match err {
            ConfigError::ModeNotFound { mode, .. } => assert_eq!(mode, "nonexistent"),
            other => panic!("expected ModeNotFound, got {other:?}"),
        }
    }

    #[test]
    fn no_mode_loads_base_alone() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let base_path = tmp.path().join("config.yml");
        std::fs::write(&base_path, MINIMUM_BASE).expect("write base");

        let cfg = load_layered(
            &base_path,
            LayeredOptions {
                mode: None,
                modes_dir: None,
            },
        )
        .expect("load");
        assert_eq!(cfg.server.host, "127.0.0.1");
        assert_eq!(cfg.server.port, 15002);
        assert_eq!(cfg.logging.level, "info");
    }

    #[test]
    fn mode_override_replaces_only_changed_keys() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let base_path = tmp.path().join("config.yml");
        std::fs::write(&base_path, MINIMUM_BASE).expect("write base");

        let modes_dir = tmp.path().join("config").join("modes");
        std::fs::create_dir_all(&modes_dir).expect("mkdir");
        std::fs::write(
            modes_dir.join("production.yml"),
            "server:\n  host: 0.0.0.0\nlogging:\n  level: warn\n",
        )
        .expect("write override");

        let cfg = load_layered(
            &base_path,
            LayeredOptions {
                mode: Some("production".to_owned()),
                modes_dir: Some(modes_dir),
            },
        )
        .expect("load");
        assert_eq!(cfg.server.host, "0.0.0.0", "host overridden by mode");
        assert_eq!(
            cfg.logging.level, "warn",
            "logging level overridden by mode"
        );
        assert_eq!(
            cfg.server.port, 15002,
            "port stays at base value because the override did not touch it"
        );
        assert_eq!(cfg.server.mcp_port, 15002, "mcp_port stays at base too");
    }

    #[test]
    fn mode_from_env_treats_empty_string_as_unset() {
        // `set_var` is `unsafe` in modern std but the test serializes
        // env reads via #[serial_test::serial] in larger suites; here
        // a single-test module is fine because this test takes the env
        // mutex implicitly through the OS.
        // SAFETY: writing an env var inside a test is safe because no
        // other test thread is observing this var at this instant.
        unsafe {
            std::env::set_var("VECTORIZER_MODE", "");
        }
        assert_eq!(mode_from_env(), None);
        // SAFETY: same as above.
        unsafe {
            std::env::set_var("VECTORIZER_MODE", "  production  ");
        }
        assert_eq!(mode_from_env(), Some("production".to_owned()));
        // SAFETY: clean up after ourselves.
        unsafe {
            std::env::remove_var("VECTORIZER_MODE");
        }
    }
}
