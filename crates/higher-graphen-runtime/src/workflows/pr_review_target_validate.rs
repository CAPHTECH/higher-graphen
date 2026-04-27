use super::{validation_error, INPUT_SCHEMA};
use crate::error::{RuntimeError, RuntimeResult};
use crate::pr_review_reports::PrReviewTargetInputDocument;
use higher_graphen_core::Id;

pub(super) fn validate_input_schema(input: &PrReviewTargetInputDocument) -> RuntimeResult<()> {
    if input.schema == INPUT_SCHEMA {
        return Ok(());
    }
    Err(RuntimeError::unsupported_input_schema(
        input.schema.clone(),
        INPUT_SCHEMA,
    ))
}

pub(super) fn validate_input_references(input: &PrReviewTargetInputDocument) -> RuntimeResult<()> {
    if input.changed_files.is_empty() {
        return Err(validation_error(
            "changed_files must contain at least one file",
        ));
    }
    ensure_unique_input_ids(input)?;
    let ids = ReferenceIds::from_input(input);
    validate_changed_files(input, &ids)?;
    validate_symbols(input, &ids)?;
    validate_owner_context_and_tests(input, &ids)?;
    validate_dependency_edges(input, &ids)?;
    validate_evidence_and_signals(input, &ids)
}

struct ReferenceIds {
    file_ids: Vec<Id>,
    symbol_ids: Vec<Id>,
    owner_ids: Vec<Id>,
    context_ids: Vec<Id>,
    test_ids: Vec<Id>,
    accepted_ids: Vec<Id>,
}

impl ReferenceIds {
    fn from_input(input: &PrReviewTargetInputDocument) -> Self {
        Self {
            file_ids: input
                .changed_files
                .iter()
                .map(|file| file.id.clone())
                .collect(),
            symbol_ids: input
                .symbols
                .iter()
                .map(|symbol| symbol.id.clone())
                .collect(),
            owner_ids: input.owners.iter().map(|owner| owner.id.clone()).collect(),
            context_ids: input
                .contexts
                .iter()
                .map(|context| context.id.clone())
                .collect(),
            test_ids: input.tests.iter().map(|test| test.id.clone()).collect(),
            accepted_ids: accepted_input_ids(input),
        }
    }

    fn dependency_endpoint_ids(&self) -> Vec<Id> {
        self.file_ids
            .iter()
            .chain(self.symbol_ids.iter())
            .chain(self.test_ids.iter())
            .chain(self.owner_ids.iter())
            .cloned()
            .collect()
    }
}

fn validate_changed_files(
    input: &PrReviewTargetInputDocument,
    ids: &ReferenceIds,
) -> RuntimeResult<()> {
    for file in &input.changed_files {
        ensure_known_ids(
            &ids.symbol_ids,
            &file.symbol_ids,
            "changed_file",
            &file.id,
            "symbol",
        )?;
        ensure_known_ids(
            &ids.owner_ids,
            &file.owner_ids,
            "changed_file",
            &file.id,
            "owner",
        )?;
        ensure_known_ids(
            &ids.context_ids,
            &file.context_ids,
            "changed_file",
            &file.id,
            "context",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &file.source_ids,
            "changed_file",
            &file.id,
            "source",
        )?;
    }
    Ok(())
}

fn validate_symbols(input: &PrReviewTargetInputDocument, ids: &ReferenceIds) -> RuntimeResult<()> {
    for symbol in &input.symbols {
        ensure_known_id(&ids.file_ids, &symbol.file_id, "symbol", &symbol.id, "file")?;
        ensure_known_ids(
            &ids.owner_ids,
            &symbol.owner_ids,
            "symbol",
            &symbol.id,
            "owner",
        )?;
        ensure_known_ids(
            &ids.context_ids,
            &symbol.context_ids,
            "symbol",
            &symbol.id,
            "context",
        )?;
    }
    Ok(())
}

fn validate_owner_context_and_tests(
    input: &PrReviewTargetInputDocument,
    ids: &ReferenceIds,
) -> RuntimeResult<()> {
    for owner in &input.owners {
        ensure_known_ids(
            &ids.accepted_ids,
            &owner.source_ids,
            "owner",
            &owner.id,
            "source",
        )?;
    }
    for context in &input.contexts {
        ensure_known_ids(
            &ids.accepted_ids,
            &context.source_ids,
            "context",
            &context.id,
            "source",
        )?;
    }
    validate_tests(input, ids)
}

fn validate_tests(input: &PrReviewTargetInputDocument, ids: &ReferenceIds) -> RuntimeResult<()> {
    for test in &input.tests {
        if let Some(file_id) = &test.file_id {
            ensure_known_id(&ids.file_ids, file_id, "test", &test.id, "file")?;
        }
        ensure_known_ids(
            &ids.symbol_ids,
            &test.symbol_ids,
            "test",
            &test.id,
            "symbol",
        )?;
        ensure_known_ids(
            &ids.context_ids,
            &test.context_ids,
            "test",
            &test.id,
            "context",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &test.source_ids,
            "test",
            &test.id,
            "source",
        )?;
    }
    Ok(())
}

fn validate_dependency_edges(
    input: &PrReviewTargetInputDocument,
    ids: &ReferenceIds,
) -> RuntimeResult<()> {
    let endpoint_ids = ids.dependency_endpoint_ids();
    for edge in &input.dependency_edges {
        ensure_known_id(
            &endpoint_ids,
            &edge.from_id,
            "dependency_edge",
            &edge.id,
            "from endpoint",
        )?;
        ensure_known_id(
            &endpoint_ids,
            &edge.to_id,
            "dependency_edge",
            &edge.id,
            "to endpoint",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &edge.source_ids,
            "dependency_edge",
            &edge.id,
            "source",
        )?;
    }
    Ok(())
}

fn validate_evidence_and_signals(
    input: &PrReviewTargetInputDocument,
    ids: &ReferenceIds,
) -> RuntimeResult<()> {
    for evidence in &input.evidence {
        ensure_known_ids(
            &ids.accepted_ids,
            &evidence.source_ids,
            "evidence",
            &evidence.id,
            "source",
        )?;
    }
    for signal in &input.signals {
        ensure_known_ids(
            &ids.accepted_ids,
            &signal.source_ids,
            "signal",
            &signal.id,
            "source",
        )?;
    }
    Ok(())
}

fn ensure_unique_input_ids(input: &PrReviewTargetInputDocument) -> RuntimeResult<()> {
    let mut seen = Vec::new();
    ensure_unique_id(&mut seen, &input.repository.id, "repository")?;
    ensure_unique_id(&mut seen, &input.pull_request.id, "pull_request")?;
    for (role, ids) in input_id_groups(input) {
        for id in ids {
            ensure_unique_id(&mut seen, &id, role)?;
        }
    }
    Ok(())
}

fn input_id_groups(input: &PrReviewTargetInputDocument) -> Vec<(&'static str, Vec<Id>)> {
    vec![
        (
            "changed_file",
            input
                .changed_files
                .iter()
                .map(|file| file.id.clone())
                .collect(),
        ),
        (
            "symbol",
            input
                .symbols
                .iter()
                .map(|symbol| symbol.id.clone())
                .collect(),
        ),
        (
            "owner",
            input.owners.iter().map(|owner| owner.id.clone()).collect(),
        ),
        (
            "context",
            input
                .contexts
                .iter()
                .map(|context| context.id.clone())
                .collect(),
        ),
        (
            "test",
            input.tests.iter().map(|test| test.id.clone()).collect(),
        ),
        (
            "dependency_edge",
            input
                .dependency_edges
                .iter()
                .map(|edge| edge.id.clone())
                .collect(),
        ),
        (
            "evidence",
            input
                .evidence
                .iter()
                .map(|evidence| evidence.id.clone())
                .collect(),
        ),
        (
            "signal",
            input
                .signals
                .iter()
                .map(|signal| signal.id.clone())
                .collect(),
        ),
    ]
}

fn ensure_unique_id(
    seen: &mut Vec<(Id, &'static str)>,
    id: &Id,
    role: &'static str,
) -> RuntimeResult<()> {
    if let Some((_, existing_role)) = seen.iter().find(|(seen_id, _)| seen_id == id) {
        return Err(validation_error(format!(
            "{role} id {id} duplicates existing {existing_role} id"
        )));
    }
    seen.push((id.clone(), role));
    Ok(())
}

fn ensure_known_ids(
    known_ids: &[Id],
    referenced_ids: &[Id],
    owner_role: &str,
    owner_id: &Id,
    referenced_role: &str,
) -> RuntimeResult<()> {
    for referenced_id in referenced_ids {
        ensure_known_id(
            known_ids,
            referenced_id,
            owner_role,
            owner_id,
            referenced_role,
        )?;
    }
    Ok(())
}

fn ensure_known_id(
    known_ids: &[Id],
    referenced_id: &Id,
    owner_role: &str,
    owner_id: &Id,
    referenced_role: &str,
) -> RuntimeResult<()> {
    if known_ids.contains(referenced_id) {
        return Ok(());
    }
    Err(validation_error(format!(
        "{owner_role} {owner_id} references unknown {referenced_role} {referenced_id}"
    )))
}

fn accepted_input_ids(input: &PrReviewTargetInputDocument) -> Vec<Id> {
    let mut ids = vec![input.repository.id.clone(), input.pull_request.id.clone()];
    ids.extend(input.changed_files.iter().map(|file| file.id.clone()));
    ids.extend(input.symbols.iter().map(|symbol| symbol.id.clone()));
    ids.extend(input.owners.iter().map(|owner| owner.id.clone()));
    ids.extend(input.contexts.iter().map(|context| context.id.clone()));
    ids.extend(input.tests.iter().map(|test| test.id.clone()));
    ids.extend(input.dependency_edges.iter().map(|edge| edge.id.clone()));
    ids.extend(input.evidence.iter().map(|evidence| evidence.id.clone()));
    ids.extend(input.signals.iter().map(|signal| signal.id.clone()));
    ids
}
