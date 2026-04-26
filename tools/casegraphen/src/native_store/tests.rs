use super::*;
use crate::native_model::{CaseMorphismType, CaseSpace};
use higher_graphen_core::{Id, ReviewStatus};
use std::{fs, path::PathBuf};

const NATIVE_EXAMPLE: &str =
    include_str!("../../../../schemas/casegraphen/native.case.space.example.json");

#[test]
fn import_list_inspect_history_and_replay_case_space() {
    let root = temp_root("round-trip");
    let store = NativeCaseStore::new(root.clone());
    let case_space = fixture_space();

    let imported = store
        .import_case_space(&case_space)
        .expect("import native case space");
    let listed = store.list_case_spaces().expect("list case spaces");
    let inspected = store
        .inspect_case_space(&case_space.case_space_id)
        .expect("inspect case space");
    let history = store
        .history_entries(&case_space.case_space_id)
        .expect("history entries");
    let replay = store
        .replay_current_case_space(&case_space.case_space_id)
        .expect("replay case space");

    assert_eq!(
        imported.current_revision_id,
        case_space.revision.revision_id
    );
    assert_eq!(listed, vec![inspected.clone()]);
    assert_eq!(inspected.history_entry_count, 1);
    assert_eq!(history, case_space.morphism_log);
    assert_eq!(replay.case_space, case_space);
    assert!(
        store
            .validate_case_space(&replay.case_space_id)
            .expect("validate case space")
            .valid
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn append_metadata_only_morphism_advances_history_and_replay() {
    let root = temp_root("append");
    let store = NativeCaseStore::new(root.clone());
    let case_space = fixture_space();
    store
        .import_case_space(&case_space)
        .expect("import native case space");
    let mut entry = case_space.morphism_log[0].clone();
    entry.sequence = 2;
    entry.entry_id = id("morphism_log_entry:metadata-only");
    entry.morphism_id = id("morphism:metadata-only");
    entry.source_revision_id = Some(case_space.revision.revision_id.clone());
    entry.target_revision_id = id("revision:native-contract-v2");
    entry.morphism.morphism_id = entry.morphism_id.clone();
    entry.morphism.morphism_type = CaseMorphismType::Review;
    entry.morphism.source_revision_id = entry.source_revision_id.clone();
    entry.morphism.target_revision_id = entry.target_revision_id.clone();
    entry.morphism.added_ids = Vec::new();
    entry.morphism.updated_ids = Vec::new();
    entry.morphism.retired_ids = Vec::new();
    entry.morphism.preserved_ids = vec![id("goal:native-case-contract")];
    entry.morphism.review_status = ReviewStatus::Reviewed;

    let mut expected = case_space.clone();
    expected.morphism_log.push(entry.clone());
    expected.revision = revision_from_entry(&expected.case_space_id, &entry);
    entry.replay_checksum = case_space_checksum(&expected).expect("checksum");

    store
        .append_morphism(&case_space.case_space_id, entry.clone())
        .expect("append metadata-only morphism");

    let replay = store
        .replay_current_case_space(&case_space.case_space_id)
        .expect("replay after append");
    assert_eq!(replay.history.len(), 2);
    assert_eq!(
        replay.current_revision_id,
        id("revision:native-contract-v2")
    );
    assert_eq!(replay.case_space.morphism_log[1], entry);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn import_rejects_unsupported_case_space_schema() {
    let root = temp_root("bad-schema");
    let store = NativeCaseStore::new(root.clone());
    let mut case_space = fixture_space();
    case_space.schema = "highergraphen.case.space.v0".to_owned();

    let error = store
        .import_case_space(&case_space)
        .expect_err("unsupported schema");
    assert!(matches!(error, NativeStoreError::UnsupportedSchema { .. }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn import_rejects_multi_entry_materialized_log_without_partial_write() {
    let root = temp_root("multi-entry-import");
    let store = NativeCaseStore::new(root.clone());
    let mut case_space = fixture_space();
    let mut second_entry = case_space.morphism_log[0].clone();
    second_entry.sequence = 2;
    second_entry.entry_id = id("morphism_log_entry:second");
    second_entry.morphism_id = id("morphism:second");
    second_entry.source_revision_id = Some(case_space.revision.revision_id.clone());
    second_entry.target_revision_id = id("revision:second");
    second_entry.morphism.morphism_id = second_entry.morphism_id.clone();
    second_entry.morphism.source_revision_id = second_entry.source_revision_id.clone();
    second_entry.morphism.target_revision_id = second_entry.target_revision_id.clone();
    case_space.morphism_log.push(second_entry);

    let error = store
        .import_case_space(&case_space)
        .expect_err("multi-entry imports are not materializable without prior snapshots");

    assert!(matches!(error, NativeStoreError::ReplayMismatch { .. }));
    assert!(!store.log_path(&case_space.case_space_id).exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn history_rejects_malformed_log_sequence() {
    let root = temp_root("bad-history");
    let store = NativeCaseStore::new(root.clone());
    let case_space = fixture_space();
    store
        .import_case_space(&case_space)
        .expect("import native case space");
    let log_path = store.log_path(&case_space.case_space_id);
    let mut bad_entry = case_space.morphism_log[0].clone();
    bad_entry.sequence = 2;
    fs::write(
        &log_path,
        format!(
            "{}\n",
            serde_json::to_string(&bad_entry).expect("serialize bad entry")
        ),
    )
    .expect("rewrite malformed log");

    let error = store
        .history_entries(&case_space.case_space_id)
        .expect_err("malformed history");
    assert!(matches!(error, NativeStoreError::ReplayMismatch { .. }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn append_rejects_unmaterialized_payload_changes() {
    let root = temp_root("bad-append");
    let store = NativeCaseStore::new(root.clone());
    let case_space = fixture_space();
    store
        .import_case_space(&case_space)
        .expect("import native case space");
    let mut entry = case_space.morphism_log[0].clone();
    entry.sequence = 2;
    entry.entry_id = id("morphism_log_entry:unsupported-payload");
    entry.morphism_id = id("morphism:unsupported-payload");
    entry.source_revision_id = Some(case_space.revision.revision_id.clone());
    entry.target_revision_id = id("revision:unsupported-payload");
    entry.morphism.morphism_id = entry.morphism_id.clone();
    entry.morphism.source_revision_id = entry.source_revision_id.clone();
    entry.morphism.target_revision_id = entry.target_revision_id.clone();
    entry.morphism.added_ids = vec![id("case:not-materialized")];

    let error = store
        .append_morphism(&case_space.case_space_id, entry)
        .expect_err("unsupported payload changes");
    assert!(matches!(error, NativeStoreError::InvalidMorphism { .. }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn append_rejects_morphism_that_does_not_advance_revision() {
    let root = temp_root("same-revision-append");
    let store = NativeCaseStore::new(root.clone());
    let case_space = fixture_space();
    store
        .import_case_space(&case_space)
        .expect("import native case space");
    let mut entry = case_space.morphism_log[0].clone();
    entry.sequence = 2;
    entry.entry_id = id("morphism_log_entry:same-revision");
    entry.morphism_id = id("morphism:same-revision");
    entry.source_revision_id = Some(case_space.revision.revision_id.clone());
    entry.target_revision_id = case_space.revision.revision_id.clone();
    entry.morphism.morphism_id = entry.morphism_id.clone();
    entry.morphism.source_revision_id = entry.source_revision_id.clone();
    entry.morphism.target_revision_id = entry.target_revision_id.clone();
    entry.morphism.added_ids = Vec::new();
    entry.morphism.updated_ids = Vec::new();
    entry.morphism.retired_ids = Vec::new();

    let error = store
        .append_morphism(&case_space.case_space_id, entry)
        .expect_err("same revision append");
    assert!(matches!(error, NativeStoreError::InvalidMorphism { .. }));

    let _ = fs::remove_dir_all(root);
}

fn fixture_space() -> CaseSpace {
    serde_json::from_str(NATIVE_EXAMPLE).expect("native case space example")
}

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "casegraphen-native-store-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    root
}

fn id(value: &str) -> Id {
    Id::new(value).expect("fixture id")
}
