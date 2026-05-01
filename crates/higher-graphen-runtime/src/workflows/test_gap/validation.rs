use super::*;

pub(super) fn validate_input_schema(input: &TestGapInputDocument) -> RuntimeResult<()> {
    if input.schema == INPUT_SCHEMA {
        return Ok(());
    }
    Err(RuntimeError::unsupported_input_schema(
        input.schema.clone(),
        INPUT_SCHEMA,
    ))
}

pub(super) fn validate_input_references(input: &TestGapInputDocument) -> RuntimeResult<()> {
    if input.changed_files.is_empty() {
        return Err(validation_error(
            "changed_files must contain at least one file",
        ));
    }
    ensure_unique_input_ids(input)?;
    let ids = ReferenceIds::from_input(input);
    validate_changed_files(input, &ids)?;
    validate_symbols(input, &ids)?;
    validate_branches(input, &ids)?;
    validate_requirements(input, &ids)?;
    validate_tests(input, &ids)?;
    validate_coverage(input, &ids)?;
    validate_dependency_edges(input, &ids)?;
    validate_higher_order_cells(input, &ids)?;
    validate_higher_order_incidences(input, &ids)?;
    validate_morphisms(input, &ids)?;
    validate_laws(input, &ids)?;
    validate_verification_cells(input, &ids)?;
    validate_source_only_records(input, &ids)
}

fn validate_changed_files(input: &TestGapInputDocument, ids: &ReferenceIds) -> RuntimeResult<()> {
    for file in &input.changed_files {
        ensure_known_ids(
            &ids.symbol_ids,
            &file.symbol_ids,
            "changed_file",
            &file.id,
            "symbol",
        )?;
        ensure_known_ids(
            &ids.context_ids,
            &file.context_ids,
            "changed_file",
            &file.id,
            "context",
        )?;
        ensure_source_ids(ids, &file.source_ids, "changed_file", &file.id)?;
    }
    Ok(())
}

fn validate_symbols(input: &TestGapInputDocument, ids: &ReferenceIds) -> RuntimeResult<()> {
    for symbol in &input.symbols {
        ensure_known_id(&ids.file_ids, &symbol.file_id, "symbol", &symbol.id, "file")?;
        ensure_known_ids(
            &ids.branch_ids,
            &symbol.branch_ids,
            "symbol",
            &symbol.id,
            "branch",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &symbol.requirement_ids,
            "symbol",
            &symbol.id,
            "requirement",
        )?;
        ensure_known_ids(
            &ids.context_ids,
            &symbol.context_ids,
            "symbol",
            &symbol.id,
            "context",
        )?;
        ensure_source_ids(ids, &symbol.source_ids, "symbol", &symbol.id)?;
    }
    Ok(())
}

fn validate_branches(input: &TestGapInputDocument, ids: &ReferenceIds) -> RuntimeResult<()> {
    for branch in &input.branches {
        ensure_known_id(
            &ids.symbol_ids,
            &branch.symbol_id,
            "branch",
            &branch.id,
            "symbol",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &branch.requirement_ids,
            "branch",
            &branch.id,
            "requirement",
        )?;
        ensure_source_ids(ids, &branch.source_ids, "branch", &branch.id)?;
    }
    Ok(())
}

fn validate_requirements(input: &TestGapInputDocument, ids: &ReferenceIds) -> RuntimeResult<()> {
    for requirement in &input.requirements {
        ensure_known_ids(
            &ids.implementation_ids,
            &requirement.implementation_ids,
            "requirement",
            &requirement.id,
            "implementation",
        )?;
        ensure_source_ids(ids, &requirement.source_ids, "requirement", &requirement.id)?;
    }
    Ok(())
}

fn validate_tests(input: &TestGapInputDocument, ids: &ReferenceIds) -> RuntimeResult<()> {
    for test in &input.tests {
        if let Some(file_id) = &test.file_id {
            ensure_known_id(&ids.file_ids, file_id, "test", &test.id, "file")?;
        }
        ensure_known_ids(
            &ids.implementation_ids,
            &test.target_ids,
            "test",
            &test.id,
            "target",
        )?;
        ensure_known_ids(
            &ids.branch_ids,
            &test.branch_ids,
            "test",
            &test.id,
            "branch",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &test.requirement_ids,
            "test",
            &test.id,
            "requirement",
        )?;
        ensure_known_ids(
            &ids.context_ids,
            &test.context_ids,
            "test",
            &test.id,
            "context",
        )?;
        ensure_source_ids(ids, &test.source_ids, "test", &test.id)?;
    }
    Ok(())
}

fn validate_coverage(input: &TestGapInputDocument, ids: &ReferenceIds) -> RuntimeResult<()> {
    for coverage in &input.coverage {
        ensure_known_id(
            &ids.coverage_target_ids,
            &coverage.target_id,
            "coverage",
            &coverage.id,
            "target",
        )?;
        ensure_known_ids(
            &ids.test_ids,
            &coverage.covered_by_test_ids,
            "coverage",
            &coverage.id,
            "test",
        )?;
        ensure_source_ids(ids, &coverage.source_ids, "coverage", &coverage.id)?;
    }
    Ok(())
}

fn validate_dependency_edges(
    input: &TestGapInputDocument,
    ids: &ReferenceIds,
) -> RuntimeResult<()> {
    for edge in &input.dependency_edges {
        ensure_known_id(
            &ids.accepted_ids,
            &edge.from_id,
            "dependency_edge",
            &edge.id,
            "from endpoint",
        )?;
        ensure_known_id(
            &ids.accepted_ids,
            &edge.to_id,
            "dependency_edge",
            &edge.id,
            "to endpoint",
        )?;
        ensure_source_ids(ids, &edge.source_ids, "dependency_edge", &edge.id)?;
    }
    Ok(())
}

fn validate_higher_order_cells(
    input: &TestGapInputDocument,
    ids: &ReferenceIds,
) -> RuntimeResult<()> {
    for cell in &input.higher_order_cells {
        ensure_known_ids(
            &ids.context_ids,
            &cell.context_ids,
            "higher_order_cell",
            &cell.id,
            "context",
        )?;
        ensure_source_ids(ids, &cell.source_ids, "higher_order_cell", &cell.id)?;
    }
    Ok(())
}

fn validate_higher_order_incidences(
    input: &TestGapInputDocument,
    ids: &ReferenceIds,
) -> RuntimeResult<()> {
    for incidence in &input.higher_order_incidences {
        ensure_known_id(
            &ids.high_order_endpoint_ids,
            &incidence.from_id,
            "higher_order_incidence",
            &incidence.id,
            "from endpoint",
        )?;
        ensure_known_id(
            &ids.high_order_endpoint_ids,
            &incidence.to_id,
            "higher_order_incidence",
            &incidence.id,
            "to endpoint",
        )?;
        ensure_source_ids(
            ids,
            &incidence.source_ids,
            "higher_order_incidence",
            &incidence.id,
        )?;
    }
    Ok(())
}

fn validate_morphisms(input: &TestGapInputDocument, ids: &ReferenceIds) -> RuntimeResult<()> {
    for morphism in &input.morphisms {
        ensure_known_ids(
            &ids.high_order_endpoint_ids,
            &morphism.source_ids,
            "morphism",
            &morphism.id,
            "source endpoint",
        )?;
        ensure_known_ids(
            &ids.high_order_endpoint_ids,
            &morphism.target_ids,
            "morphism",
            &morphism.id,
            "target endpoint",
        )?;
        ensure_known_ids(
            &ids.law_ids,
            &morphism.law_ids,
            "morphism",
            &morphism.id,
            "law",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &morphism.requirement_ids,
            "morphism",
            &morphism.id,
            "requirement",
        )?;
    }
    Ok(())
}

fn validate_laws(input: &TestGapInputDocument, ids: &ReferenceIds) -> RuntimeResult<()> {
    for law in &input.laws {
        ensure_known_ids(
            &ids.high_order_endpoint_ids,
            &law.applies_to_ids,
            "law",
            &law.id,
            "applies-to endpoint",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &law.requirement_ids,
            "law",
            &law.id,
            "requirement",
        )?;
        ensure_source_ids(ids, &law.source_ids, "law", &law.id)?;
    }
    Ok(())
}

fn validate_verification_cells(
    input: &TestGapInputDocument,
    ids: &ReferenceIds,
) -> RuntimeResult<()> {
    for verification in &input.verification_cells {
        ensure_known_ids(
            &ids.high_order_endpoint_ids,
            &verification.target_ids,
            "verification_cell",
            &verification.id,
            "target",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &verification.requirement_ids,
            "verification_cell",
            &verification.id,
            "requirement",
        )?;
        ensure_known_ids(
            &ids.law_ids,
            &verification.law_ids,
            "verification_cell",
            &verification.id,
            "law",
        )?;
        ensure_known_ids(
            &ids.morphism_ids,
            &verification.morphism_ids,
            "verification_cell",
            &verification.id,
            "morphism",
        )?;
        ensure_source_ids(
            ids,
            &verification.source_ids,
            "verification_cell",
            &verification.id,
        )?;
    }
    Ok(())
}

fn validate_source_only_records(
    input: &TestGapInputDocument,
    ids: &ReferenceIds,
) -> RuntimeResult<()> {
    for context in &input.contexts {
        ensure_source_ids(ids, &context.source_ids, "context", &context.id)?;
    }
    for evidence in &input.evidence {
        ensure_source_ids(ids, &evidence.source_ids, "evidence", &evidence.id)?;
    }
    for signal in &input.signals {
        ensure_source_ids(ids, &signal.source_ids, "signal", &signal.id)?;
    }
    Ok(())
}

fn ensure_source_ids(
    ids: &ReferenceIds,
    source_ids: &[Id],
    owner_kind: &str,
    owner_id: &Id,
) -> RuntimeResult<()> {
    ensure_known_ids(
        &ids.accepted_ids,
        source_ids,
        owner_kind,
        owner_id,
        "source",
    )
}

pub(super) struct ReferenceIds {
    file_ids: Vec<Id>,
    symbol_ids: Vec<Id>,
    branch_ids: Vec<Id>,
    requirement_ids: Vec<Id>,
    test_ids: Vec<Id>,
    law_ids: Vec<Id>,
    morphism_ids: Vec<Id>,
    context_ids: Vec<Id>,
    implementation_ids: Vec<Id>,
    coverage_target_ids: Vec<Id>,
    high_order_endpoint_ids: Vec<Id>,
    accepted_ids: Vec<Id>,
}

impl ReferenceIds {
    fn from_input(input: &TestGapInputDocument) -> Self {
        let file_ids = input
            .changed_files
            .iter()
            .map(|file| file.id.clone())
            .collect();
        let symbol_ids = input
            .symbols
            .iter()
            .map(|symbol| symbol.id.clone())
            .collect();
        let branch_ids = input
            .branches
            .iter()
            .map(|branch| branch.id.clone())
            .collect();
        let requirement_ids = input
            .requirements
            .iter()
            .map(|requirement| requirement.id.clone())
            .collect();
        let test_ids = input.tests.iter().map(|test| test.id.clone()).collect();
        let law_ids = input.laws.iter().map(|law| law.id.clone()).collect();
        let morphism_ids = input
            .morphisms
            .iter()
            .map(|morphism| morphism.id.clone())
            .collect();
        let context_ids = input
            .contexts
            .iter()
            .map(|context| context.id.clone())
            .collect();
        let mut implementation_ids = Vec::new();
        implementation_ids.extend(input.changed_files.iter().map(|file| file.id.clone()));
        implementation_ids.extend(input.symbols.iter().map(|symbol| symbol.id.clone()));
        implementation_ids.extend(input.higher_order_cells.iter().map(|cell| cell.id.clone()));
        implementation_ids.extend(input.laws.iter().map(|law| law.id.clone()));
        implementation_ids.extend(input.morphisms.iter().map(|morphism| morphism.id.clone()));
        let mut coverage_target_ids = implementation_ids.clone();
        coverage_target_ids.extend(input.branches.iter().map(|branch| branch.id.clone()));
        coverage_target_ids.extend(
            input
                .requirements
                .iter()
                .map(|requirement| requirement.id.clone()),
        );
        let mut high_order_endpoint_ids = accepted_fact_ids(input);
        high_order_endpoint_ids.extend(input.higher_order_cells.iter().map(|cell| cell.id.clone()));
        high_order_endpoint_ids.extend(input.laws.iter().map(|law| law.id.clone()));
        high_order_endpoint_ids.extend(input.morphisms.iter().map(|morphism| morphism.id.clone()));
        Self {
            file_ids,
            symbol_ids,
            branch_ids,
            requirement_ids,
            test_ids,
            law_ids,
            morphism_ids,
            context_ids,
            implementation_ids,
            coverage_target_ids,
            high_order_endpoint_ids,
            accepted_ids: accepted_fact_ids(input),
        }
    }
}

pub(super) fn ensure_unique_input_ids(input: &TestGapInputDocument) -> RuntimeResult<()> {
    let mut seen: BTreeSet<Id> = BTreeSet::new();
    ensure_globally_unique_input_ids(input, &mut seen)?;
    ensure_kind_local_unique_input_ids(input)
}

fn ensure_globally_unique_input_ids(
    input: &TestGapInputDocument,
    seen: &mut BTreeSet<Id>,
) -> RuntimeResult<()> {
    ensure_global_ids(
        seen,
        "changed_file",
        input.changed_files.iter().map(|file| file.id.clone()),
    )?;
    ensure_global_ids(
        seen,
        "symbol",
        input.symbols.iter().map(|symbol| symbol.id.clone()),
    )?;
    ensure_global_ids(
        seen,
        "branch",
        input.branches.iter().map(|branch| branch.id.clone()),
    )?;
    ensure_global_ids(
        seen,
        "requirement",
        input.requirements.iter().map(|item| item.id.clone()),
    )?;
    ensure_global_ids(seen, "test", input.tests.iter().map(|test| test.id.clone()))?;
    ensure_global_ids(
        seen,
        "coverage",
        input.coverage.iter().map(|item| item.id.clone()),
    )?;
    ensure_global_ids(
        seen,
        "dependency_edge",
        input.dependency_edges.iter().map(|item| item.id.clone()),
    )?;
    ensure_global_ids(
        seen,
        "higher_order_incidence",
        input
            .higher_order_incidences
            .iter()
            .map(|item| item.id.clone()),
    )?;
    ensure_global_ids(
        seen,
        "morphism",
        input.morphisms.iter().map(|item| item.id.clone()),
    )?;
    ensure_global_ids(
        seen,
        "verification_cell",
        input.verification_cells.iter().map(|item| item.id.clone()),
    )?;
    ensure_global_ids(
        seen,
        "context",
        input.contexts.iter().map(|item| item.id.clone()),
    )?;
    ensure_global_ids(
        seen,
        "evidence",
        input.evidence.iter().map(|item| item.id.clone()),
    )?;
    ensure_global_ids(
        seen,
        "signal",
        input.signals.iter().map(|item| item.id.clone()),
    )?;
    Ok(())
}

fn ensure_kind_local_unique_input_ids(input: &TestGapInputDocument) -> RuntimeResult<()> {
    ensure_unique_ids_within_kind(
        "higher_order_cell",
        input
            .higher_order_cells
            .iter()
            .map(|cell| cell.id.clone())
            .collect(),
    )?;
    ensure_unique_ids_within_kind("law", input.laws.iter().map(|law| law.id.clone()).collect())?;
    Ok(())
}

fn ensure_global_ids(
    seen: &mut BTreeSet<Id>,
    kind: &str,
    ids: impl IntoIterator<Item = Id>,
) -> RuntimeResult<()> {
    for id in ids {
        if !seen.insert(id.clone()) {
            return Err(validation_error(format!(
                "{kind} id {id} duplicates existing input id"
            )));
        }
    }
    Ok(())
}

pub(super) fn ensure_unique_ids_within_kind(kind: &str, ids: Vec<Id>) -> RuntimeResult<()> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id.clone()) {
            return Err(validation_error(format!(
                "{kind} id {id} duplicates existing {kind} id"
            )));
        }
    }
    Ok(())
}

pub(super) fn ensure_known_ids(
    known_ids: &[Id],
    referenced_ids: &[Id],
    owner_kind: &str,
    owner_id: &Id,
    referenced_kind: &str,
) -> RuntimeResult<()> {
    for referenced_id in referenced_ids {
        ensure_known_id(
            known_ids,
            referenced_id,
            owner_kind,
            owner_id,
            referenced_kind,
        )?;
    }
    Ok(())
}

pub(super) fn ensure_known_id(
    known_ids: &[Id],
    referenced_id: &Id,
    owner_kind: &str,
    owner_id: &Id,
    referenced_kind: &str,
) -> RuntimeResult<()> {
    if known_ids.contains(referenced_id) {
        return Ok(());
    }
    Err(validation_error(format!(
        "{owner_kind} {owner_id} references unknown {referenced_kind} {referenced_id}"
    )))
}
