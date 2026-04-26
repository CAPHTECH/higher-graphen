use crate::native_model::{
    CaseSpace, MorphismLogEntry, Revision, NATIVE_CASE_SPACE_SCHEMA,
    NATIVE_CASE_SPACE_SCHEMA_VERSION, NATIVE_MORPHISM_LOG_ENTRY_SCHEMA,
};
use higher_graphen_core::Id;
use serde_json::Map;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

mod support;
mod types;
use support::*;
pub use types::*;

const NATIVE_DIRECTORY: &str = "native_case_spaces";

impl NativeCaseStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn import_case_space(
        &self,
        case_space: &CaseSpace,
    ) -> NativeStoreResult<NativeCaseSpaceRecord> {
        require_case_space_contract(&self.root, case_space)?;
        require_importable_materialized_log(&self.root, case_space)?;
        validate_materialized_log(&self.root, case_space)?;

        let case_dir = self.case_dir(&case_space.case_space_id);
        let snapshots_dir = case_dir.join("snapshots");
        fs::create_dir_all(&snapshots_dir).map_err(|source| NativeStoreError::Io {
            path: snapshots_dir.clone(),
            source,
        })?;

        let log_path = self.log_path(&case_space.case_space_id);
        let mut snapshot = case_space.clone();
        snapshot.morphism_log = case_space.morphism_log.clone();
        write_json(
            &self.resolve_snapshot_path(
                &self.relative_snapshot_path(
                    &case_space.case_space_id,
                    &case_space.revision.revision_id,
                ),
                &log_path,
            )?,
            &snapshot,
        )?;

        fs::write(&log_path, "").map_err(|source| NativeStoreError::Io {
            path: log_path.clone(),
            source,
        })?;
        for entry in &case_space.morphism_log {
            append_json_line(&log_path, entry)?;
        }

        self.inspect_case_space(&case_space.case_space_id)
    }

    pub fn append_morphism(
        &self,
        case_space_id: &Id,
        entry: MorphismLogEntry,
    ) -> NativeStoreResult<NativeCaseSpaceRecord> {
        let replay = self.replay_current_case_space(case_space_id)?;
        let log_path = self.log_path(case_space_id);
        validate_append(&log_path, &replay.case_space, &entry, &replay.history)?;

        let mut next = replay.case_space;
        apply_bounded_morphism(&log_path, &mut next, &entry)?;
        next.morphism_log.push(entry.clone());
        next.revision = revision_from_entry(&next.case_space_id, &entry);

        let expected_checksum = case_space_checksum(&next)?;
        if entry.replay_checksum != expected_checksum {
            return Err(NativeStoreError::ReplayMismatch {
                path: log_path,
                reason: format!(
                    "entry {} replay_checksum {} does not match computed {}",
                    entry.entry_id, entry.replay_checksum, expected_checksum
                ),
            });
        }

        let snapshot_path = self.resolve_snapshot_path(
            &self.relative_snapshot_path(&next.case_space_id, &next.revision.revision_id),
            &self.log_path(case_space_id),
        )?;
        write_json(&snapshot_path, &next)?;
        append_json_line(&self.log_path(case_space_id), &entry)?;
        self.inspect_case_space(case_space_id)
    }

    pub fn list_case_spaces(&self) -> NativeStoreResult<Vec<NativeCaseSpaceRecord>> {
        let root = self.native_root();
        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => return Err(NativeStoreError::Io { path: root, source }),
        };

        let mut directories = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| NativeStoreError::Io {
                path: root.clone(),
                source,
            })?;
            if entry
                .file_type()
                .map_err(|source| NativeStoreError::Io {
                    path: entry.path(),
                    source,
                })?
                .is_dir()
            {
                directories.push(entry.path());
            }
        }
        directories.sort();

        let mut records = Vec::new();
        for directory in directories {
            records.push(self.inspect_directory(&directory)?);
        }
        records.sort_by_key(|record| record.case_space_id.as_str().to_owned());
        Ok(records)
    }

    pub fn inspect_case_space(
        &self,
        case_space_id: &Id,
    ) -> NativeStoreResult<NativeCaseSpaceRecord> {
        let entries = self.history_entries(case_space_id)?;
        native_record(self, case_space_id, &entries)
    }

    pub fn history_entries(&self, case_space_id: &Id) -> NativeStoreResult<Vec<MorphismLogEntry>> {
        let path = self.log_path(case_space_id);
        if !path.exists() {
            return Err(NativeStoreError::MissingCase {
                case_space_id: case_space_id.clone(),
                path,
            });
        }
        let text = fs::read_to_string(&path).map_err(|source| NativeStoreError::Io {
            path: path.clone(),
            source,
        })?;
        let entries = parse_log_entries(&path, &text)?;
        validate_log_entries(self, Some(case_space_id), &path, &entries)?;
        Ok(entries)
    }

    pub fn replay_current_case_space(
        &self,
        case_space_id: &Id,
    ) -> NativeStoreResult<NativeCaseSpaceReplay> {
        let entries = self.history_entries(case_space_id)?;
        let latest = latest_entry(&entries, &self.log_path(case_space_id))?;
        let snapshot_path = self.resolve_snapshot_path(
            &self.relative_snapshot_path(&latest.case_space_id, &latest.target_revision_id),
            &self.log_path(case_space_id),
        )?;
        let case_space = read_case_space(&snapshot_path)?;
        require_case_space_matches_entry(&snapshot_path, &case_space, latest)?;
        validate_materialized_log(&snapshot_path, &case_space)?;

        Ok(NativeCaseSpaceReplay {
            schema: NATIVE_CASE_SPACE_REPLAY_SCHEMA.to_owned(),
            schema_version: NATIVE_STORE_SCHEMA_VERSION,
            case_space_id: latest.case_space_id.clone(),
            space_id: case_space.space_id.clone(),
            current_revision_id: latest.target_revision_id.clone(),
            case_space,
            history: entries,
        })
    }

    pub fn validate_case_space(
        &self,
        case_space_id: &Id,
    ) -> NativeStoreResult<NativeCaseSpaceValidation> {
        let replay = self.replay_current_case_space(case_space_id)?;
        Ok(NativeCaseSpaceValidation {
            schema: NATIVE_CASE_SPACE_VALIDATION_SCHEMA.to_owned(),
            schema_version: NATIVE_STORE_SCHEMA_VERSION,
            case_space_id: replay.case_space_id,
            current_revision_id: replay.current_revision_id,
            history_entry_count: replay.history.len() as u32,
            valid: true,
        })
    }

    fn inspect_directory(&self, directory: &Path) -> NativeStoreResult<NativeCaseSpaceRecord> {
        let log_path = directory.join("morphism_log.jsonl");
        let text = fs::read_to_string(&log_path).map_err(|source| NativeStoreError::Io {
            path: log_path.clone(),
            source,
        })?;
        let entries = parse_log_entries(&log_path, &text)?;
        validate_log_entries(self, None, &log_path, &entries)?;
        let latest = latest_entry(&entries, &log_path)?;
        native_record(self, &latest.case_space_id, &entries)
    }

    fn native_root(&self) -> PathBuf {
        self.root.join(NATIVE_DIRECTORY)
    }

    fn case_dir(&self, case_space_id: &Id) -> PathBuf {
        self.native_root().join(path_segment(case_space_id))
    }

    fn log_path(&self, case_space_id: &Id) -> PathBuf {
        self.case_dir(case_space_id).join("morphism_log.jsonl")
    }

    fn relative_snapshot_path(&self, case_space_id: &Id, revision_id: &Id) -> String {
        format!(
            "{}/{}/snapshots/{}.case.space.json",
            NATIVE_DIRECTORY,
            path_segment(case_space_id),
            path_segment(revision_id)
        )
    }

    fn resolve_snapshot_path(
        &self,
        relative_path: &str,
        log_path: &Path,
    ) -> NativeStoreResult<PathBuf> {
        require_relative_store_path(log_path, relative_path)?;
        Ok(self.root.join(relative_path))
    }
}

fn native_record(
    store: &NativeCaseStore,
    case_space_id: &Id,
    entries: &[MorphismLogEntry],
) -> NativeStoreResult<NativeCaseSpaceRecord> {
    let latest = latest_entry(entries, &store.native_root())?;
    if &latest.case_space_id != case_space_id {
        return Err(NativeStoreError::ReplayMismatch {
            path: store.native_root(),
            reason: format!(
                "history for {case_space_id} ended with case space {}",
                latest.case_space_id
            ),
        });
    }
    let current_snapshot_path =
        store.relative_snapshot_path(case_space_id, &latest.target_revision_id);
    let current_snapshot = read_case_space(
        &store.resolve_snapshot_path(&current_snapshot_path, &store.log_path(case_space_id))?,
    )?;
    require_case_space_matches_entry(&store.log_path(case_space_id), &current_snapshot, latest)?;

    let revisions = entries
        .iter()
        .map(|entry| NativeRevisionRecord {
            revision_id: entry.target_revision_id.clone(),
            parent_revision_id: entry.source_revision_id.clone(),
            sequence: entry.sequence,
            entry_id: entry.entry_id.clone(),
            morphism_id: entry.morphism_id.clone(),
            snapshot_path: store.relative_snapshot_path(case_space_id, &entry.target_revision_id),
            source_ids: entry.source_ids.clone(),
            replay_checksum: entry.replay_checksum.clone(),
        })
        .collect::<Vec<_>>();

    Ok(NativeCaseSpaceRecord {
        schema: NATIVE_CASE_SPACE_RECORD_SCHEMA.to_owned(),
        schema_version: NATIVE_STORE_SCHEMA_VERSION,
        case_space_id: latest.case_space_id.clone(),
        space_id: current_snapshot.space_id,
        current_revision_id: latest.target_revision_id.clone(),
        case_space_directory: format!("{}/{}", NATIVE_DIRECTORY, path_segment(case_space_id)),
        log_path: format!(
            "{}/{}/morphism_log.jsonl",
            NATIVE_DIRECTORY,
            path_segment(case_space_id)
        ),
        current_snapshot_path,
        revision_count: revisions.len() as u32,
        history_entry_count: entries.len() as u32,
        revisions,
    })
}

fn validate_append(
    path: &Path,
    current: &CaseSpace,
    entry: &MorphismLogEntry,
    existing_entries: &[MorphismLogEntry],
) -> NativeStoreResult<()> {
    require_log_entry_contract(path, entry)?;
    require_entry_morphism_match(path, entry)?;
    if entry.case_space_id != current.case_space_id {
        return Err(invalid_morphism(
            path,
            format!(
                "entry case_space_id {} does not match {}",
                entry.case_space_id, current.case_space_id
            ),
        ));
    }
    if entry.sequence != existing_entries.len() as u64 + 1 {
        return Err(invalid_morphism(
            path,
            format!("entry sequence must be {}", existing_entries.len() + 1),
        ));
    }
    if entry.source_revision_id.as_ref() != Some(&current.revision.revision_id) {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: format!(
                "entry source_revision_id {:?} does not match current revision {}",
                entry.source_revision_id, current.revision.revision_id
            ),
        });
    }
    if entry.target_revision_id == current.revision.revision_id {
        return Err(invalid_morphism(
            path,
            "entry target_revision_id must advance the revision",
        ));
    }
    if existing_entries
        .iter()
        .any(|existing| existing.entry_id == entry.entry_id)
    {
        return Err(invalid_morphism(
            path,
            format!("duplicate log entry {}", entry.entry_id),
        ));
    }
    if existing_entries
        .iter()
        .any(|existing| existing.morphism_id == entry.morphism_id)
    {
        return Err(invalid_morphism(
            path,
            format!("duplicate morphism {}", entry.morphism_id),
        ));
    }
    Ok(())
}

fn validate_log_entries(
    store: &NativeCaseStore,
    expected_case_space_id: Option<&Id>,
    path: &Path,
    entries: &[MorphismLogEntry],
) -> NativeStoreResult<()> {
    if entries.is_empty() {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: "morphism log is empty".to_owned(),
        });
    }

    let mut seen_entries = BTreeSet::new();
    let mut seen_morphisms = BTreeSet::new();
    let mut previous_revision_id: Option<Id> = None;
    for (index, entry) in entries.iter().enumerate() {
        require_log_entry_contract(path, entry)?;
        require_entry_morphism_match(path, entry)?;
        if let Some(expected_id) = expected_case_space_id {
            if &entry.case_space_id != expected_id {
                return Err(NativeStoreError::ReplayMismatch {
                    path: path.to_owned(),
                    reason: format!(
                        "log entry {} belongs to {}, expected {}",
                        entry.entry_id, entry.case_space_id, expected_id
                    ),
                });
            }
        }
        if entry.sequence != index as u64 + 1 {
            return Err(NativeStoreError::ReplayMismatch {
                path: path.to_owned(),
                reason: format!(
                    "log entry {} has sequence {}, expected {}",
                    entry.entry_id,
                    entry.sequence,
                    index + 1
                ),
            });
        }
        if index == 0 {
            if entry.source_revision_id.is_some() {
                return Err(NativeStoreError::ReplayMismatch {
                    path: path.to_owned(),
                    reason: "first morphism log entry must not set source_revision_id".to_owned(),
                });
            }
        } else if entry.source_revision_id.as_ref() != previous_revision_id.as_ref() {
            return Err(NativeStoreError::ReplayMismatch {
                path: path.to_owned(),
                reason: format!(
                    "log entry {} has source_revision_id {:?}, expected {:?}",
                    entry.entry_id, entry.source_revision_id, previous_revision_id
                ),
            });
        }
        if !seen_entries.insert(entry.entry_id.clone()) {
            return Err(invalid_morphism(
                path,
                format!("duplicate log entry {}", entry.entry_id),
            ));
        }
        if !seen_morphisms.insert(entry.morphism_id.clone()) {
            return Err(invalid_morphism(
                path,
                format!("duplicate morphism {}", entry.morphism_id),
            ));
        }
        let snapshot_path = store.resolve_snapshot_path(
            &store.relative_snapshot_path(&entry.case_space_id, &entry.target_revision_id),
            path,
        )?;
        let snapshot = read_case_space(&snapshot_path)?;
        require_case_space_matches_entry(&snapshot_path, &snapshot, entry)?;
        previous_revision_id = Some(entry.target_revision_id.clone());
    }
    Ok(())
}

fn require_importable_materialized_log(
    path: &Path,
    case_space: &CaseSpace,
) -> NativeStoreResult<()> {
    if case_space.morphism_log.len() != 1 {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: "native import requires a single materialized genesis log entry; append later morphisms through the native store".to_owned(),
        });
    }

    let entry = &case_space.morphism_log[0];
    require_log_entry_contract(path, entry)?;
    require_entry_morphism_match(path, entry)?;
    if entry.sequence != 1 {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: format!(
                "first morphism log entry has sequence {}, expected 1",
                entry.sequence
            ),
        });
    }
    if entry.source_revision_id.is_some() {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: "first morphism log entry must not set source_revision_id".to_owned(),
        });
    }
    Ok(())
}

fn validate_materialized_log(path: &Path, case_space: &CaseSpace) -> NativeStoreResult<()> {
    if case_space.morphism_log.is_empty() {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: "case space morphism_log is empty".to_owned(),
        });
    }
    let latest = case_space
        .morphism_log
        .last()
        .expect("empty log checked before latest access");
    if case_space.revision.case_space_id != case_space.case_space_id {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: format!(
                "revision case_space_id {} does not match {}",
                case_space.revision.case_space_id, case_space.case_space_id
            ),
        });
    }
    if latest.case_space_id != case_space.case_space_id {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: format!(
                "latest log case_space_id {} does not match {}",
                latest.case_space_id, case_space.case_space_id
            ),
        });
    }
    if latest.target_revision_id != case_space.revision.revision_id {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: format!(
                "latest log target_revision_id {} does not match revision {}",
                latest.target_revision_id, case_space.revision.revision_id
            ),
        });
    }
    if latest.replay_checksum != case_space.revision.checksum {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: format!(
                "latest replay_checksum {} does not match revision checksum {}",
                latest.replay_checksum, case_space.revision.checksum
            ),
        });
    }
    require_ids_exist(path, case_space)?;
    Ok(())
}

fn apply_bounded_morphism(
    path: &Path,
    case_space: &mut CaseSpace,
    entry: &MorphismLogEntry,
) -> NativeStoreResult<()> {
    let morphism = &entry.morphism;
    if !morphism.added_ids.is_empty()
        || !morphism.updated_ids.is_empty()
        || !morphism.retired_ids.is_empty()
    {
        return Err(invalid_morphism(
            path,
            "native store replay only accepts metadata-only morphisms until typed reducers exist",
        ));
    }
    require_referenced_ids_exist(path, case_space, &morphism.preserved_ids)?;
    require_referenced_ids_exist(path, case_space, &morphism.evidence_ids)?;
    Ok(())
}

fn revision_from_entry(case_space_id: &Id, entry: &MorphismLogEntry) -> Revision {
    Revision {
        revision_id: entry.target_revision_id.clone(),
        case_space_id: case_space_id.clone(),
        applied_entry_ids: vec![entry.entry_id.clone()],
        applied_morphism_ids: vec![entry.morphism_id.clone()],
        checksum: entry.replay_checksum.clone(),
        parent_revision_id: entry.source_revision_id.clone(),
        created_at: entry.recorded_at.clone(),
        source_ids: entry.source_ids.clone(),
        metadata: Map::new(),
    }
}

fn require_case_space_contract(path: &Path, case_space: &CaseSpace) -> NativeStoreResult<()> {
    if case_space.schema != NATIVE_CASE_SPACE_SCHEMA {
        return Err(NativeStoreError::UnsupportedSchema {
            path: path.to_owned(),
            actual: case_space.schema.clone(),
            expected: NATIVE_CASE_SPACE_SCHEMA,
        });
    }
    if case_space.schema_version != NATIVE_CASE_SPACE_SCHEMA_VERSION {
        return Err(NativeStoreError::UnsupportedVersion {
            path: path.to_owned(),
            actual: case_space.schema_version,
            expected: NATIVE_CASE_SPACE_SCHEMA_VERSION,
        });
    }
    Ok(())
}

fn require_log_entry_contract(path: &Path, entry: &MorphismLogEntry) -> NativeStoreResult<()> {
    if entry.schema != NATIVE_MORPHISM_LOG_ENTRY_SCHEMA {
        return Err(NativeStoreError::UnsupportedSchema {
            path: path.to_owned(),
            actual: entry.schema.clone(),
            expected: NATIVE_MORPHISM_LOG_ENTRY_SCHEMA,
        });
    }
    if entry.schema_version != NATIVE_CASE_SPACE_SCHEMA_VERSION {
        return Err(NativeStoreError::UnsupportedVersion {
            path: path.to_owned(),
            actual: entry.schema_version,
            expected: NATIVE_CASE_SPACE_SCHEMA_VERSION,
        });
    }
    Ok(())
}

fn require_entry_morphism_match(path: &Path, entry: &MorphismLogEntry) -> NativeStoreResult<()> {
    if entry.morphism_id != entry.morphism.morphism_id {
        return Err(invalid_morphism(
            path,
            format!(
                "entry morphism_id {} does not match payload {}",
                entry.morphism_id, entry.morphism.morphism_id
            ),
        ));
    }
    if entry.source_revision_id != entry.morphism.source_revision_id {
        return Err(invalid_morphism(
            path,
            "entry source_revision_id does not match morphism payload",
        ));
    }
    if entry.target_revision_id != entry.morphism.target_revision_id {
        return Err(invalid_morphism(
            path,
            "entry target_revision_id does not match morphism payload",
        ));
    }
    Ok(())
}

fn require_case_space_matches_entry(
    path: &Path,
    case_space: &CaseSpace,
    entry: &MorphismLogEntry,
) -> NativeStoreResult<()> {
    require_case_space_contract(path, case_space)?;
    if case_space.case_space_id != entry.case_space_id {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: format!(
                "case_space_id {} does not match log entry {}",
                case_space.case_space_id, entry.case_space_id
            ),
        });
    }
    if case_space.revision.revision_id != entry.target_revision_id {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: format!(
                "revision {} does not match log target {}",
                case_space.revision.revision_id, entry.target_revision_id
            ),
        });
    }
    if case_space.revision.checksum != entry.replay_checksum {
        return Err(NativeStoreError::ReplayMismatch {
            path: path.to_owned(),
            reason: format!(
                "revision checksum {} does not match replay checksum {}",
                case_space.revision.checksum, entry.replay_checksum
            ),
        });
    }
    Ok(())
}

fn require_ids_exist(path: &Path, case_space: &CaseSpace) -> NativeStoreResult<()> {
    let ids = known_ids(case_space);
    for relation in &case_space.case_relations {
        require_referenced_ids(
            path,
            &ids,
            &[relation.from_id.clone(), relation.to_id.clone()],
        )?;
        require_referenced_ids(path, &ids, &relation.evidence_ids)?;
    }
    for projection in &case_space.projections {
        require_referenced_ids(path, &ids, &projection.represented_cell_ids)?;
        require_referenced_ids(path, &ids, &projection.represented_relation_ids)?;
        require_referenced_ids(path, &ids, &projection.omitted_cell_ids)?;
        require_referenced_ids(path, &ids, &projection.omitted_relation_ids)?;
    }
    Ok(())
}

fn require_referenced_ids_exist(
    path: &Path,
    case_space: &CaseSpace,
    references: &[Id],
) -> NativeStoreResult<()> {
    require_referenced_ids(path, &known_ids(case_space), references)
}

fn require_referenced_ids(
    path: &Path,
    ids: &BTreeSet<Id>,
    references: &[Id],
) -> NativeStoreResult<()> {
    for id in references {
        if !ids.contains(id) {
            return Err(invalid_morphism(
                path,
                format!("unknown referenced id {id}"),
            ));
        }
    }
    Ok(())
}

fn known_ids(case_space: &CaseSpace) -> BTreeSet<Id> {
    case_space
        .case_cells
        .iter()
        .map(|cell| cell.id.clone())
        .chain(
            case_space
                .case_relations
                .iter()
                .map(|relation| relation.id.clone()),
        )
        .chain(
            case_space
                .projections
                .iter()
                .map(|projection| projection.projection_id.clone()),
        )
        .chain(
            case_space
                .morphism_log
                .iter()
                .flat_map(|entry| [entry.entry_id.clone(), entry.morphism_id.clone()]),
        )
        .chain([case_space.revision.revision_id.clone()])
        .collect()
}

fn read_case_space(path: &Path) -> NativeStoreResult<CaseSpace> {
    let text = fs::read_to_string(path).map_err(|source| NativeStoreError::Io {
        path: path.to_owned(),
        source,
    })?;
    let case_space: CaseSpace =
        serde_json::from_str(&text).map_err(|source| NativeStoreError::Json {
            path: path.to_owned(),
            source,
        })?;
    require_case_space_contract(path, &case_space)?;
    Ok(case_space)
}

#[cfg(test)]
mod tests;
