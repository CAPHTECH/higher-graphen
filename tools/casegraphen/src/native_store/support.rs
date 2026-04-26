use super::{CaseSpace, MorphismLogEntry, NativeStoreError, NativeStoreResult};
use higher_graphen_core::Id;
use serde::Serialize;
use serde_json::Value;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};

pub(super) fn parse_log_entries(
    path: &Path,
    text: &str,
) -> NativeStoreResult<Vec<MorphismLogEntry>> {
    let mut entries = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        entries.push(
            serde_json::from_str(line).map_err(|source| NativeStoreError::Json {
                path: path.to_owned(),
                source,
            })?,
        );
    }
    Ok(entries)
}

pub(super) fn append_json_line(path: &Path, value: &impl Serialize) -> NativeStoreResult<()> {
    let text = serde_json::to_string(value).map_err(|source| NativeStoreError::Json {
        path: path.to_owned(),
        source,
    })?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| NativeStoreError::Io {
            path: path.to_owned(),
            source,
        })?;
    writeln!(file, "{text}").map_err(|source| NativeStoreError::Io {
        path: path.to_owned(),
        source,
    })
}

pub(super) fn write_json(path: &Path, value: &impl Serialize) -> NativeStoreResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| NativeStoreError::Io {
            path: parent.to_owned(),
            source,
        })?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|source| NativeStoreError::Json {
        path: path.to_owned(),
        source,
    })?;
    fs::write(path, format!("{text}\n")).map_err(|source| NativeStoreError::Io {
        path: path.to_owned(),
        source,
    })
}

pub(super) fn latest_entry<'a>(
    entries: &'a [MorphismLogEntry],
    path: &Path,
) -> NativeStoreResult<&'a MorphismLogEntry> {
    entries
        .last()
        .ok_or_else(|| NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: "morphism log is empty".to_owned(),
        })
}

pub(super) fn require_relative_store_path(path: &Path, value: &str) -> NativeStoreResult<()> {
    let candidate = Path::new(value);
    if value.trim().is_empty() {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: "snapshot path is empty".to_owned(),
        });
    }
    for component in candidate.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(NativeStoreError::ReplayMismatch {
                path: path.to_owned(),
                reason: format!("snapshot path {value:?} must stay inside the native store"),
            });
        }
    }
    Ok(())
}

pub(super) fn case_space_checksum(case_space: &CaseSpace) -> NativeStoreResult<String> {
    let mut value = serde_json::to_value(case_space).map_err(|source| NativeStoreError::Json {
        path: PathBuf::from("<case-space-checksum>"),
        source,
    })?;
    if let Value::Object(object) = &mut value {
        if let Some(Value::Object(revision)) = object.get_mut("revision") {
            revision.insert("checksum".to_owned(), Value::String(String::new()));
        }
        if let Some(Value::Array(log)) = object.get_mut("morphism_log") {
            for entry in log {
                if let Value::Object(entry) = entry {
                    entry.insert("replay_checksum".to_owned(), Value::String(String::new()));
                }
            }
        }
    }
    let canonical = serde_json::to_string(&value).map_err(|source| NativeStoreError::Json {
        path: PathBuf::from("<case-space-checksum>"),
        source,
    })?;
    Ok(format!("fnv1a64:{:016x}", fnv1a64(canonical.as_bytes())))
}

pub(super) fn path_segment(id: &Id) -> String {
    let mut segment = String::new();
    for byte in id.as_str().bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' => {
                segment.push(byte as char);
            }
            _ => segment.push_str(&format!("~{byte:02x}")),
        }
    }
    segment
}

pub(super) fn invalid_morphism(path: &Path, reason: impl Into<String>) -> NativeStoreError {
    NativeStoreError::InvalidMorphism {
        path: path.to_owned(),
        reason: reason.into(),
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
