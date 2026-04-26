use crate::native_model::{CaseSpace, MorphismLogEntry};
use higher_graphen_core::Id;
use serde::Serialize;
use std::path::PathBuf;

pub type NativeStoreResult<T> = Result<T, NativeStoreError>;

pub const NATIVE_CASE_SPACE_RECORD_SCHEMA: &str = "highergraphen.case.native_store.record.v1";
pub const NATIVE_CASE_SPACE_REPLAY_SCHEMA: &str = "highergraphen.case.native_store.replay.v1";
pub const NATIVE_CASE_SPACE_VALIDATION_SCHEMA: &str =
    "highergraphen.case.native_store.validation.v1";
pub const NATIVE_STORE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug)]
pub enum NativeStoreError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    UnsupportedSchema {
        path: PathBuf,
        actual: String,
        expected: &'static str,
    },
    UnsupportedVersion {
        path: PathBuf,
        actual: u32,
        expected: u32,
    },
    MissingCase {
        case_space_id: Id,
        path: PathBuf,
    },
    ReplayMismatch {
        path: PathBuf,
        reason: String,
    },
    InvalidMorphism {
        path: PathBuf,
        reason: String,
    },
}

pub struct NativeCaseStore {
    pub(crate) root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCaseSpaceRecord {
    pub schema: String,
    pub schema_version: u32,
    pub case_space_id: Id,
    pub space_id: Id,
    pub current_revision_id: Id,
    pub case_space_directory: String,
    pub log_path: String,
    pub current_snapshot_path: String,
    pub revision_count: u32,
    pub history_entry_count: u32,
    pub revisions: Vec<NativeRevisionRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRevisionRecord {
    pub revision_id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_revision_id: Option<Id>,
    pub sequence: u64,
    pub entry_id: Id,
    pub morphism_id: Id,
    pub snapshot_path: String,
    pub source_ids: Vec<Id>,
    pub replay_checksum: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCaseSpaceReplay {
    pub schema: String,
    pub schema_version: u32,
    pub case_space_id: Id,
    pub space_id: Id,
    pub current_revision_id: Id,
    pub case_space: CaseSpace,
    pub history: Vec<MorphismLogEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCaseSpaceValidation {
    pub schema: String,
    pub schema_version: u32,
    pub case_space_id: Id,
    pub current_revision_id: Id,
    pub history_entry_count: u32,
    pub valid: bool,
}

impl std::fmt::Display for NativeStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Json { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::UnsupportedSchema {
                path,
                actual,
                expected,
            } => write!(
                formatter,
                "{}: unsupported schema {actual:?}; expected {expected:?}",
                path.display()
            ),
            Self::UnsupportedVersion {
                path,
                actual,
                expected,
            } => write!(
                formatter,
                "{}: unsupported schema version {actual}; expected {expected}",
                path.display()
            ),
            Self::MissingCase {
                case_space_id,
                path,
            } => write!(
                formatter,
                "{}: missing native case space {case_space_id}",
                path.display()
            ),
            Self::ReplayMismatch { path, reason } => {
                write!(formatter, "{}: {reason}", path.display())
            }
            Self::InvalidMorphism { path, reason } => {
                write!(formatter, "{}: {reason}", path.display())
            }
        }
    }
}

impl std::error::Error for NativeStoreError {}
