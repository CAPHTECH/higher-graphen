use super::{NativeEvalError, NativeEvalResult, NativeEvalViolation, NativeEvalViolationCode};
use crate::native_model::{
    CaseMorphismType, CaseSpace, NATIVE_CASE_SPACE_SCHEMA, NATIVE_CASE_SPACE_SCHEMA_VERSION,
    NATIVE_MORPHISM_LOG_ENTRY_SCHEMA,
};
use higher_graphen_core::Id;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub fn validate_native_case_space(case_space: &CaseSpace) -> NativeEvalResult<()> {
    let mut violations = Vec::new();
    validate_schema(case_space, &mut violations);
    validate_source_boundary(case_space, &mut violations);
    let ids = collect_declared_ids(case_space, &mut violations);
    validate_relation_references(case_space, &ids, &mut violations);
    validate_projection_references(case_space, &ids, &mut violations);
    validate_morphism_log(case_space, &ids, &mut violations);

    if violations.is_empty() {
        Ok(())
    } else {
        Err(NativeEvalError { violations })
    }
}

fn collect_declared_ids(
    case_space: &CaseSpace,
    violations: &mut Vec<NativeEvalViolation>,
) -> BTreeSet<Id> {
    let mut ids = BTreeSet::new();
    for cell in &case_space.case_cells {
        insert_id(&mut ids, violations, &cell.id, "case_cell");
        validate_cell(case_space, cell, violations);
    }
    for relation in &case_space.case_relations {
        insert_id(&mut ids, violations, &relation.id, "case_relation");
    }
    for projection in &case_space.projections {
        insert_id(
            &mut ids,
            violations,
            &projection.projection_id,
            "projection",
        );
    }
    insert_id(
        &mut ids,
        violations,
        &case_space.revision.revision_id,
        "revision",
    );
    for entry in &case_space.morphism_log {
        insert_id(&mut ids, violations, &entry.entry_id, "morphism_log_entry");
        insert_id(&mut ids, violations, &entry.morphism_id, "morphism");
        ids.insert(entry.target_revision_id.clone());
        if let Some(source_revision_id) = &entry.source_revision_id {
            ids.insert(source_revision_id.clone());
        }
    }
    ids
}

fn validate_cell(
    case_space: &CaseSpace,
    cell: &crate::native_model::CaseCell,
    violations: &mut Vec<NativeEvalViolation>,
) {
    if cell.title.trim().is_empty() {
        push_violation(
            violations,
            NativeEvalViolationCode::EmptyRequiredField,
            Some(&cell.id),
            "title",
            "case cell title must not be empty",
        );
    }
    if cell.space_id != case_space.space_id {
        push_violation(
            violations,
            NativeEvalViolationCode::SpaceMismatch,
            Some(&cell.id),
            "space_id",
            format!(
                "case cell {} belongs to {}, but case space belongs to {}",
                cell.id, cell.space_id, case_space.space_id
            ),
        );
    }
}

fn validate_relation_references(
    case_space: &CaseSpace,
    ids: &BTreeSet<Id>,
    violations: &mut Vec<NativeEvalViolation>,
) {
    for relation in &case_space.case_relations {
        require_id(ids, violations, &relation.id, "from_id", &relation.from_id);
        require_id(ids, violations, &relation.id, "to_id", &relation.to_id);
        for evidence_id in &relation.evidence_ids {
            require_id(ids, violations, &relation.id, "evidence_ids", evidence_id);
        }
    }
}

fn validate_projection_references(
    case_space: &CaseSpace,
    ids: &BTreeSet<Id>,
    violations: &mut Vec<NativeEvalViolation>,
) {
    for projection in &case_space.projections {
        require_id(
            ids,
            violations,
            &projection.projection_id,
            "projection.revision_id",
            &projection.revision_id,
        );
        for cell_id in projection
            .represented_cell_ids
            .iter()
            .chain(&projection.omitted_cell_ids)
        {
            require_id(
                ids,
                violations,
                &projection.projection_id,
                "projection_cell_ids",
                cell_id,
            );
        }
        for relation_id in projection
            .represented_relation_ids
            .iter()
            .chain(&projection.omitted_relation_ids)
        {
            require_id(
                ids,
                violations,
                &projection.projection_id,
                "projection_relation_ids",
                relation_id,
            );
        }
        for loss_id in projection
            .information_loss
            .iter()
            .flat_map(|loss| loss.represented_ids.iter().chain(&loss.omitted_ids))
        {
            require_id(
                ids,
                violations,
                &projection.projection_id,
                "projection.information_loss.ids",
                loss_id,
            );
        }
    }
}

fn validate_schema(case_space: &CaseSpace, violations: &mut Vec<NativeEvalViolation>) {
    if case_space.schema != NATIVE_CASE_SPACE_SCHEMA {
        push_violation(
            violations,
            NativeEvalViolationCode::SchemaMismatch,
            Some(&case_space.case_space_id),
            "schema",
            format!(
                "unsupported native case schema {:?}; expected {:?}",
                case_space.schema, NATIVE_CASE_SPACE_SCHEMA
            ),
        );
    }
    if case_space.schema_version != NATIVE_CASE_SPACE_SCHEMA_VERSION {
        push_violation(
            violations,
            NativeEvalViolationCode::UnsupportedSchemaVersion,
            Some(&case_space.case_space_id),
            "schema_version",
            format!(
                "unsupported native case schema version {}; expected {}",
                case_space.schema_version, NATIVE_CASE_SPACE_SCHEMA_VERSION
            ),
        );
    }
}

fn validate_source_boundary(case_space: &CaseSpace, violations: &mut Vec<NativeEvalViolation>) {
    validate_source_boundary_value(
        case_space.metadata.get("source_boundary"),
        &case_space.case_space_id,
        "metadata.source_boundary",
        violations,
    );
}

fn validate_morphism_log(
    case_space: &CaseSpace,
    ids: &BTreeSet<Id>,
    violations: &mut Vec<NativeEvalViolation>,
) {
    if case_space.morphism_log.is_empty() {
        push_violation(
            violations,
            NativeEvalViolationCode::InvalidMorphismLog,
            Some(&case_space.revision.revision_id),
            "morphism_log",
            "case space morphism_log must not be empty",
        );
    }
    let mut expected_sequence = 1;
    let mut previous_target_revision_id = None::<Id>;
    for entry in &case_space.morphism_log {
        validate_log_entry_contract(case_space, entry, expected_sequence, violations);
        validate_morphism_contract(entry, previous_target_revision_id.as_ref(), violations);
        for changed_id in entry
            .morphism
            .added_ids
            .iter()
            .chain(&entry.morphism.updated_ids)
            .chain(&entry.morphism.retired_ids)
            .chain(&entry.morphism.preserved_ids)
            .chain(&entry.morphism.evidence_ids)
        {
            require_id(ids, violations, &entry.entry_id, "morphism.ids", changed_id);
        }
        expected_sequence += 1;
        previous_target_revision_id = Some(entry.target_revision_id.clone());
    }
    if let Some(last_revision_id) = previous_target_revision_id {
        validate_materialized_revision(case_space, &last_revision_id, violations);
    }
}

fn validate_log_entry_contract(
    case_space: &CaseSpace,
    entry: &crate::native_model::MorphismLogEntry,
    expected_sequence: u64,
    violations: &mut Vec<NativeEvalViolation>,
) {
    if entry.schema != NATIVE_MORPHISM_LOG_ENTRY_SCHEMA {
        push_violation(
            violations,
            NativeEvalViolationCode::InvalidMorphismLog,
            Some(&entry.entry_id),
            "schema",
            "morphism log entry schema mismatch",
        );
    }
    if entry.schema_version != NATIVE_CASE_SPACE_SCHEMA_VERSION {
        push_violation(
            violations,
            NativeEvalViolationCode::UnsupportedSchemaVersion,
            Some(&entry.entry_id),
            "schema_version",
            format!(
                "unsupported morphism log schema version {}; expected {}",
                entry.schema_version, NATIVE_CASE_SPACE_SCHEMA_VERSION
            ),
        );
    }
    if entry.case_space_id != case_space.case_space_id {
        push_violation(
            violations,
            NativeEvalViolationCode::InvalidMorphismLog,
            Some(&entry.entry_id),
            "case_space_id",
            "morphism log entry belongs to a different case space",
        );
    }
    if entry.sequence != expected_sequence {
        push_violation(
            violations,
            NativeEvalViolationCode::InvalidMorphismLog,
            Some(&entry.entry_id),
            "sequence",
            format!(
                "morphism log sequence {} should be {}",
                entry.sequence, expected_sequence
            ),
        );
    }
}

fn validate_morphism_contract(
    entry: &crate::native_model::MorphismLogEntry,
    previous_target_revision_id: Option<&Id>,
    violations: &mut Vec<NativeEvalViolation>,
) {
    compare_morphism_field(
        entry,
        "morphism_id",
        entry.morphism_id == entry.morphism.morphism_id,
        "entry morphism_id must match nested morphism.morphism_id",
        violations,
    );
    compare_morphism_field(
        entry,
        "source_revision_id",
        entry.source_revision_id == entry.morphism.source_revision_id,
        "entry source_revision_id must match nested morphism.source_revision_id",
        violations,
    );
    compare_morphism_field(
        entry,
        "target_revision_id",
        entry.target_revision_id == entry.morphism.target_revision_id,
        "entry target_revision_id must match nested morphism.target_revision_id",
        violations,
    );
    if previous_target_revision_id.is_none() && entry.source_revision_id.is_some() {
        push_violation(
            violations,
            NativeEvalViolationCode::InvalidMorphismLog,
            Some(&entry.entry_id),
            "source_revision_id",
            "first morphism log entry must not set source_revision_id",
        );
    }
    if previous_target_revision_id.is_some()
        && entry.source_revision_id.as_ref() != previous_target_revision_id
    {
        push_violation(
            violations,
            NativeEvalViolationCode::InvalidMorphismLog,
            Some(&entry.entry_id),
            "source_revision_id",
            "morphism source revision must equal the previous log target revision",
        );
    }
    if previous_target_revision_id.is_none() {
        validate_lift_morphism_metadata(entry, violations);
    }
}

fn validate_lift_morphism_metadata(
    entry: &crate::native_model::MorphismLogEntry,
    violations: &mut Vec<NativeEvalViolation>,
) {
    let is_lift_like = match &entry.morphism.morphism_type {
        CaseMorphismType::Create | CaseMorphismType::Migration => true,
        CaseMorphismType::Custom(extension) => extension == "lift",
        _ => false,
    };
    if !is_lift_like {
        push_violation(
            violations,
            NativeEvalViolationCode::InvalidMorphism,
            Some(&entry.entry_id),
            "morphism.morphism_type",
            "first native morphism must be create, migration, or custom:lift",
        );
    }
    let lift_semantics = entry
        .morphism
        .metadata
        .get("lift_semantics")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    if !lift_semantics {
        push_violation(
            violations,
            NativeEvalViolationCode::EmptyRequiredField,
            Some(&entry.entry_id),
            "morphism.metadata.lift_semantics",
            "first native morphism must declare lift_semantics",
        );
    }
    validate_source_boundary_value(
        entry.morphism.metadata.get("source_boundary"),
        &entry.entry_id,
        "morphism.metadata.source_boundary",
        violations,
    );
}

fn validate_source_boundary_value(
    value: Option<&Value>,
    record_id: &Id,
    field_prefix: &str,
    violations: &mut Vec<NativeEvalViolation>,
) {
    let Some(value) = value else {
        push_violation(
            violations,
            NativeEvalViolationCode::EmptyRequiredField,
            Some(record_id),
            field_prefix,
            "native case spaces must declare a bounded source boundary",
        );
        return;
    };
    let Some(boundary) = value.as_object() else {
        push_violation(
            violations,
            NativeEvalViolationCode::EmptyRequiredField,
            Some(record_id),
            field_prefix,
            "source boundary must be an object",
        );
        return;
    };

    require_id_string(boundary, "id", record_id, field_prefix, violations);
    require_non_empty_array(
        boundary,
        "included_sources",
        record_id,
        field_prefix,
        violations,
    );
    require_non_empty_array(boundary, "adapters", record_id, field_prefix, violations);
    require_non_empty_string(
        boundary,
        "accepted_fact_policy",
        record_id,
        field_prefix,
        violations,
    );
    require_non_empty_string(
        boundary,
        "inference_policy",
        record_id,
        field_prefix,
        violations,
    );
    require_array(
        boundary,
        "information_loss",
        record_id,
        field_prefix,
        violations,
    );
}

fn require_non_empty_array(
    object: &Map<String, Value>,
    key: &str,
    record_id: &Id,
    field_prefix: &str,
    violations: &mut Vec<NativeEvalViolation>,
) {
    if !object
        .get(key)
        .and_then(Value::as_array)
        .is_some_and(|values| !values.is_empty())
    {
        push_violation(
            violations,
            NativeEvalViolationCode::EmptyRequiredField,
            Some(record_id),
            format!("{field_prefix}.{key}"),
            "source boundary field must be a non-empty array",
        );
    }
}

fn require_array(
    object: &Map<String, Value>,
    key: &str,
    record_id: &Id,
    field_prefix: &str,
    violations: &mut Vec<NativeEvalViolation>,
) {
    if !object.get(key).and_then(Value::as_array).is_some() {
        push_violation(
            violations,
            NativeEvalViolationCode::EmptyRequiredField,
            Some(record_id),
            format!("{field_prefix}.{key}"),
            "source boundary field must be an array",
        );
    }
}

fn require_non_empty_string(
    object: &Map<String, Value>,
    key: &str,
    record_id: &Id,
    field_prefix: &str,
    violations: &mut Vec<NativeEvalViolation>,
) {
    if !object
        .get(key)
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
    {
        push_violation(
            violations,
            NativeEvalViolationCode::EmptyRequiredField,
            Some(record_id),
            format!("{field_prefix}.{key}"),
            "source boundary field must be a non-empty string",
        );
    }
}

fn require_id_string(
    object: &Map<String, Value>,
    key: &str,
    record_id: &Id,
    field_prefix: &str,
    violations: &mut Vec<NativeEvalViolation>,
) {
    let valid = object
        .get(key)
        .and_then(Value::as_str)
        .is_some_and(|value| Id::new(value.to_owned()).is_ok());
    if !valid {
        push_violation(
            violations,
            NativeEvalViolationCode::EmptyRequiredField,
            Some(record_id),
            format!("{field_prefix}.{key}"),
            "source boundary field must be a valid id string",
        );
    }
}

fn compare_morphism_field(
    entry: &crate::native_model::MorphismLogEntry,
    field: &str,
    valid: bool,
    message: &str,
    violations: &mut Vec<NativeEvalViolation>,
) {
    if !valid {
        push_violation(
            violations,
            NativeEvalViolationCode::InvalidMorphism,
            Some(&entry.entry_id),
            field,
            message,
        );
    }
}

fn validate_materialized_revision(
    case_space: &CaseSpace,
    last_revision_id: &Id,
    violations: &mut Vec<NativeEvalViolation>,
) {
    if case_space.revision.revision_id != *last_revision_id {
        push_violation(
            violations,
            NativeEvalViolationCode::InvalidMorphismLog,
            Some(&case_space.revision.revision_id),
            "revision.revision_id",
            "materialized revision must match the latest morphism log target revision",
        );
    }
    if case_space.revision.case_space_id != case_space.case_space_id {
        push_violation(
            violations,
            NativeEvalViolationCode::InvalidMorphismLog,
            Some(&case_space.revision.revision_id),
            "revision.case_space_id",
            "materialized revision must belong to the case space",
        );
    }
    if let Some(latest) = case_space.morphism_log.last() {
        if case_space.revision.checksum != latest.replay_checksum {
            push_violation(
                violations,
                NativeEvalViolationCode::InvalidMorphismLog,
                Some(&case_space.revision.revision_id),
                "revision.checksum",
                "materialized revision checksum must match the latest morphism replay checksum",
            );
        }
    }
}

fn insert_id(
    ids: &mut BTreeSet<Id>,
    violations: &mut Vec<NativeEvalViolation>,
    id: &Id,
    record_type: &str,
) {
    if !ids.insert(id.clone()) {
        push_violation(
            violations,
            NativeEvalViolationCode::DuplicateId,
            Some(id),
            "id",
            format!("duplicate native record id {id} appears while inserting {record_type}"),
        );
    }
}

fn require_id(
    ids: &BTreeSet<Id>,
    violations: &mut Vec<NativeEvalViolation>,
    record_id: &Id,
    field: &str,
    target_id: &Id,
) {
    if !ids.contains(target_id) {
        push_violation(
            violations,
            NativeEvalViolationCode::DanglingReference,
            Some(record_id),
            field,
            format!("{field} references missing id {target_id}"),
        );
    }
}

fn push_violation(
    violations: &mut Vec<NativeEvalViolation>,
    code: NativeEvalViolationCode,
    record_id: Option<&Id>,
    field: impl Into<String>,
    message: impl Into<String>,
) {
    violations.push(NativeEvalViolation {
        code,
        record_id: record_id.cloned(),
        field: field.into(),
        message: message.into(),
    });
}
