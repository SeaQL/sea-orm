//! Shared helper for invoking user crates via `cargo run` and parsing their
//! JSON API responses.

use std::{
    error::Error,
    process::{Command, Stdio},
};

use serde::Deserialize;
use serde::de::DeserializeOwned;

// ---------------------------------------------------------------------------
// Mirror of sea_orm_migration::response types
// We re-declare them here so sea-orm-cli does not need to depend on
// sea-orm-migration as a library — the contract is the JSON wire format.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub error: Option<String>,
    pub meta: ApiMeta,
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct ApiMeta {
    pub version: String,
    pub migrations_hash: Option<String>,
    pub schema_hash: Option<String>,
}

// --- entity-first ---

#[derive(Debug, Deserialize)]
pub struct DiffData {
    pub changes: Vec<String>,
    pub statements: Vec<String>,
    pub warnings: Vec<WarningJson>,
    pub suggestions: Vec<SuggestionJson>,
    pub unresolved: Vec<UnresolvedRenameJson>,
    pub schema_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateData {
    pub migration_name: String,
    pub filepath: String,
    pub changes: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WarningJson {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SuggestionJson {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnresolvedRenameJson {
    pub table: String,
    pub removed: String,
    pub candidates: Vec<String>,
}

/// Output of `schema` — entity-defined schema as SQL DDL, no DB connection needed.
#[derive(Debug, Deserialize)]
pub struct SchemaData {
    pub statements: Vec<String>,
}

// --- migration-first ---

#[derive(Debug, Deserialize)]
pub struct StatusData {
    pub migrations: Vec<MigrationEntry>,
}

#[derive(Debug, Deserialize)]
pub struct MigrationEntry {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct AppliedData {
    pub applied: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RolledBackData {
    pub rolled_back: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LifecycleData {
    pub rolled_back: Vec<String>,
    pub applied: Vec<String>,
}

/// Build the manifest path from a crate root directory.
pub fn manifest_path(dir: &str) -> String {
    if dir.ends_with('/') {
        format!("{dir}Cargo.toml")
    } else {
        format!("{dir}/Cargo.toml")
    }
}

/// Error from calling a user crate subprocess.
#[derive(Debug)]
pub enum SubprocessError {
    /// The cargo invocation itself failed (non-zero exit, or IO error).
    Spawn(String),
    /// The subprocess produced no output on stdout.
    NoOutput,
    /// stdout was not valid JSON.
    InvalidJson(String),
    /// The JSON parsed but `ok = false`.
    ApiError(ApiMeta, String),
    /// `ok = true` but `data` was null.
    MissingData,
    /// The API version returned by the subprocess does not match what we expect.
    VersionMismatch { expected: String, got: String },
}

impl std::fmt::Display for SubprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(e) => write!(f, "Failed to run subprocess: {e}"),
            Self::NoOutput => write!(f, "Subprocess produced no output"),
            Self::InvalidJson(e) => write!(f, "Invalid JSON from subprocess: {e}"),
            Self::ApiError(_, msg) => write!(f, "API error: {msg}"),
            Self::MissingData => write!(f, "API returned ok=true but no data"),
            Self::VersionMismatch { expected, got } => write!(
                f,
                "Version mismatch: CLI expects {expected}, subprocess returned {got}. \
                 Rebuild your crate with the matching sea-orm-migration version."
            ),
        }
    }
}

impl Error for SubprocessError {}

const EXPECTED_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run `cargo run --manifest-path <manifest> -- <args>` with the given env
/// vars, capture stdout, parse as `ApiResponse<T>`, and return the data.
///
/// Performs a version check on `meta.version` and returns
/// [`SubprocessError::VersionMismatch`] if they differ.
pub fn run_subprocess_json<T: DeserializeOwned>(
    manifest: &str,
    args: &[&str],
    env_database_url: Option<&str>,
    env_database_schema: Option<&str>,
) -> Result<(ApiMeta, T), SubprocessError> {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--manifest-path", manifest, "--"]);
    cmd.args(args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());

    if let Some(url) = env_database_url {
        cmd.env("DATABASE_URL", url);
    }
    if let Some(schema) = env_database_schema {
        cmd.env("DATABASE_SCHEMA", schema);
    }

    if true {
        // Append to any existing RUSTFLAGS rather than clobbering them
        let existing = std::env::var("RUSTFLAGS").unwrap_or_default();
        let new_flags = if existing.is_empty() {
            "-A warnings".to_string()
        } else {
            format!("{existing} -A warnings")
        };
        cmd.env("RUSTFLAGS", new_flags);
    }

    let output = cmd
        .output()
        .map_err(|e| SubprocessError::Spawn(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().last().unwrap_or("").trim();

    if line.is_empty() {
        return Err(SubprocessError::NoOutput);
    }

    // Parse the envelope first (without T) to get meta even on error
    let raw: serde_json::Value =
        serde_json::from_str(line).map_err(|e| SubprocessError::InvalidJson(e.to_string()))?;

    let meta_val = raw.get("meta").cloned().unwrap_or(serde_json::Value::Null);
    let meta: ApiMeta = serde_json::from_value(meta_val)
        .map_err(|e| SubprocessError::InvalidJson(e.to_string()))?;

    // Version check
    if meta.version != EXPECTED_VERSION {
        return Err(SubprocessError::VersionMismatch {
            expected: EXPECTED_VERSION.to_string(),
            got: meta.version,
        });
    }

    let ok = raw.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    if !ok {
        let error = raw
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error")
            .to_string();
        return Err(SubprocessError::ApiError(meta, error));
    }

    let data_val = raw.get("data").cloned().unwrap_or(serde_json::Value::Null);
    let data: T = serde_json::from_value(data_val)
        .map_err(|e| SubprocessError::InvalidJson(e.to_string()))?;

    Ok((meta, data))
}
