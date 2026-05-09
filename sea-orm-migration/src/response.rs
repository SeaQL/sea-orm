//! JSON response types for the sea-orm-migration machine API.
//!
//! Every command writes exactly one JSON object to stdout. The envelope is
//! [`ApiResponse`] which carries [`ApiMeta`] for version/sync tracking plus a
//! command-specific `data` payload. On error, `data` is `null` and `error`
//! contains the message.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Envelope
// ---------------------------------------------------------------------------

/// Emitted for every command. Serialized as a single JSON line to stdout.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub meta: ApiMeta,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(meta: ApiMeta, data: T) -> Self {
        Self {
            ok: true,
            error: None,
            meta,
            data: Some(data),
        }
    }

    pub fn err(meta: ApiMeta, error: impl Into<String>) -> ApiResponse<T> {
        ApiResponse {
            ok: false,
            error: Some(error.into()),
            meta,
            data: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Meta
// ---------------------------------------------------------------------------

/// Versioning and sync-tracking fields present in every response.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiMeta {
    /// Semver of the sea-orm-migration crate that produced this response.
    pub version: String,

    /// For migration-first commands: FNV64 hex digest of the sorted list of
    /// migration names registered in the binary. Changes whenever migrations
    /// are added or removed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migrations_hash: Option<String>,

    /// For entity-first commands: FNV64 hex digest of the SQL statements
    /// produced by the entity set's schema builder (backend-independent
    /// representation). Changes whenever entity definitions change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_hash: Option<String>,
}

/// Compute a stable hex string from an iterator of string slices.
/// Uses FNV-1a 64-bit — no extra dependency, deterministic, fast.
pub fn fnv64_hex<'a>(items: impl Iterator<Item = &'a str>) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;
    let mut hash = OFFSET;
    for item in items {
        for byte in item.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(PRIME);
        }
        // Separator so ["ab","c"] != ["a","bc"]
        hash ^= 0xff;
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}

// ---------------------------------------------------------------------------
// Entity-first data types
// ---------------------------------------------------------------------------

/// Output of `diff` — discovered schema changes, never writes anything.
#[derive(Debug, Serialize, Deserialize)]
pub struct DiffData {
    /// Human-readable summaries of each SQL statement that would be generated.
    pub changes: Vec<String>,
    /// Raw SQL statements that would be applied.
    pub statements: Vec<String>,
    /// Always-on warnings requiring manual attention.
    pub warnings: Vec<WarningJson>,
    /// Heuristic suggestions (renames etc.).
    pub suggestions: Vec<SuggestionJson>,
    /// Ambiguous renames that the caller must resolve before calling `generate`.
    pub unresolved: Vec<UnresolvedRenameJson>,
    /// FNV64 hex digest of the discovered SQL — must be passed back to `generate`
    /// unchanged so stale calls are rejected.
    pub schema_hash: String,
}

/// Output of `schema` — entity-defined schema as SQL DDL, no DB connection needed.
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaData {
    /// SQL DDL statements for all registered entities (CREATE TABLE, CREATE TYPE, CREATE INDEX).
    pub statements: Vec<String>,
}

/// Output of `generate` — writes a migration file and updates `lib.rs`.
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateData {
    pub migration_name: String,
    pub filepath: String,
    pub changes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WarningJson {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestionJson {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnresolvedRenameJson {
    pub table: String,
    pub removed: String,
    pub candidates: Vec<String>,
}

// ---------------------------------------------------------------------------
// Migration-first data types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationEntry {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusData {
    pub migrations: Vec<MigrationEntry>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppliedData {
    pub applied: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RolledBackData {
    pub rolled_back: Vec<String>,
}

/// Used for fresh/refresh/reset where we report both applied and rolled-back.
#[derive(Debug, Serialize, Deserialize)]
pub struct LifecycleData {
    pub rolled_back: Vec<String>,
    pub applied: Vec<String>,
}
