use super::{id, push_unique, slug, space_id, EXTRACTION_METHOD};
use crate::error::RuntimeResult;
use crate::pr_review_reports::{
    PrReviewTargetInputDocument, PrReviewTargetInputOwner, PrReviewTargetInputSymbol,
    PrReviewTargetInputTest, PrReviewTargetLiftedCell, PrReviewTargetLiftedContext,
    PrReviewTargetLiftedIncidence, PrReviewTargetLiftedSpace, PrReviewTargetLiftedStructure,
};
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, SourceRef};
use higher_graphen_structure::space::IncidenceOrientation;

pub(super) struct LiftedPrReviewTarget {
    pub(super) structure: PrReviewTargetLiftedStructure,
}

pub(super) fn lift_input(
    input: &PrReviewTargetInputDocument,
) -> RuntimeResult<LiftedPrReviewTarget> {
    let space_id = space_id(input)?;
    let context_ids = effective_context_ids(input)?;
    let contexts = lifted_contexts(input, &space_id, &context_ids);
    let cells = lifted_cells(input, &space_id, &context_ids)?;
    let incidences = lifted_incidences(input, &space_id)?;
    let space = PrReviewTargetLiftedSpace {
        id: space_id.clone(),
        name: format!("PR {} review target space", input.pull_request.number),
        description: Some(
            "Bounded structural view of changed files, symbols, ownership, tests, dependencies, evidence, signals, and review context."
                .to_owned(),
        ),
        cell_ids: cells.iter().map(|cell| cell.id.clone()).collect(),
        incidence_ids: incidences
            .iter()
            .map(|incidence| incidence.id.clone())
            .collect(),
        context_ids,
    };

    Ok(LiftedPrReviewTarget {
        structure: PrReviewTargetLiftedStructure {
            space,
            contexts,
            cells,
            incidences,
        },
    })
}

fn lifted_contexts(
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    context_ids: &[Id],
) -> Vec<PrReviewTargetLiftedContext> {
    context_ids
        .iter()
        .enumerate()
        .map(|(index, context_id)| lifted_context(input, space_id, context_id, index))
        .collect()
}

fn lifted_context(
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    context_id: &Id,
    _index: usize,
) -> PrReviewTargetLiftedContext {
    if let Some(context) = input
        .contexts
        .iter()
        .find(|context| &context.id == context_id)
    {
        return PrReviewTargetLiftedContext {
            id: context.id.clone(),
            space_id: space_id.clone(),
            name: context.name.clone(),
            context_type: serde_plain_context_type(context.context_type),
            provenance: fact_provenance(input, input.source.confidence, Some("contexts")),
        };
    }
    PrReviewTargetLiftedContext {
        id: context_id.clone(),
        space_id: space_id.clone(),
        name: format!("PR {}", input.pull_request.number),
        context_type: "pull_request".to_owned(),
        provenance: fact_provenance(input, input.source.confidence, Some("pull_request")),
    }
}

fn lifted_cells(
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
) -> RuntimeResult<Vec<PrReviewTargetLiftedCell>> {
    let mut cells = Vec::new();
    append_file_cells(&mut cells, input, space_id, default_context_ids);
    append_symbol_cells(&mut cells, input, space_id, default_context_ids);
    append_owner_cells(&mut cells, input, space_id, default_context_ids);
    append_test_cells(&mut cells, input, space_id, default_context_ids);
    append_evidence_cells(&mut cells, input, space_id, default_context_ids);
    append_signal_cells(&mut cells, input, space_id, default_context_ids);
    Ok(cells)
}

fn append_file_cells(
    cells: &mut Vec<PrReviewTargetLiftedCell>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
) {
    cells.extend(
        input
            .changed_files
            .iter()
            .map(|file| PrReviewTargetLiftedCell {
                id: file.id.clone(),
                space_id: space_id.clone(),
                dimension: 0,
                cell_type: "pr.changed_file".to_owned(),
                label: file_label(&file.path),
                context_ids: contexts_or_default(&file.context_ids, default_context_ids),
                provenance: fact_provenance(input, input.source.confidence, Some("changed_files")),
            }),
    );
}

fn append_symbol_cells(
    cells: &mut Vec<PrReviewTargetLiftedCell>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
) {
    cells.extend(input.symbols.iter().map(|symbol| PrReviewTargetLiftedCell {
        id: symbol.id.clone(),
        space_id: space_id.clone(),
        dimension: 0,
        cell_type: "pr.symbol".to_owned(),
        label: symbol.name.clone(),
        context_ids: contexts_or_default(&symbol_context_ids(input, symbol), default_context_ids),
        provenance: fact_provenance(input, input.source.confidence, Some("symbols")),
    }));
}

fn append_owner_cells(
    cells: &mut Vec<PrReviewTargetLiftedCell>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
) {
    cells.extend(input.owners.iter().map(|owner| PrReviewTargetLiftedCell {
        id: owner.id.clone(),
        space_id: space_id.clone(),
        dimension: 0,
        cell_type: "pr.owner".to_owned(),
        label: owner.name.clone().unwrap_or_else(|| owner.id.to_string()),
        context_ids: contexts_or_default(&owner_context_ids(input, owner), default_context_ids),
        provenance: fact_provenance(input, input.source.confidence, Some("owners")),
    }));
}

fn append_test_cells(
    cells: &mut Vec<PrReviewTargetLiftedCell>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
) {
    cells.extend(input.tests.iter().map(|test| PrReviewTargetLiftedCell {
        id: test.id.clone(),
        space_id: space_id.clone(),
        dimension: 0,
        cell_type: "pr.test".to_owned(),
        label: test.name.clone(),
        context_ids: contexts_or_default(&test_context_ids(input, test), default_context_ids),
        provenance: fact_provenance(input, input.source.confidence, Some("tests")),
    }));
}

fn append_evidence_cells(
    cells: &mut Vec<PrReviewTargetLiftedCell>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
) {
    cells.extend(
        input
            .evidence
            .iter()
            .map(|evidence| PrReviewTargetLiftedCell {
                id: evidence.id.clone(),
                space_id: space_id.clone(),
                dimension: 1,
                cell_type: "pr.evidence".to_owned(),
                label: evidence.summary.clone(),
                context_ids: contexts_or_default(default_context_ids, default_context_ids),
                provenance: fact_provenance(
                    input,
                    evidence.confidence.unwrap_or(input.source.confidence),
                    Some("evidence"),
                ),
            }),
    );
}

fn append_signal_cells(
    cells: &mut Vec<PrReviewTargetLiftedCell>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
) {
    cells.extend(input.signals.iter().map(|signal| PrReviewTargetLiftedCell {
        id: signal.id.clone(),
        space_id: space_id.clone(),
        dimension: 1,
        cell_type: "pr.risk_signal".to_owned(),
        label: signal.summary.clone(),
        context_ids: contexts_or_default(default_context_ids, default_context_ids),
        provenance: fact_provenance(input, signal.confidence, Some("signals")),
    }));
}

fn lifted_incidences(
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
) -> RuntimeResult<Vec<PrReviewTargetLiftedIncidence>> {
    let mut incidences = Vec::new();
    append_file_incidences(&mut incidences, input, space_id)?;
    append_symbol_incidences(&mut incidences, input, space_id)?;
    append_test_incidences(&mut incidences, input, space_id)?;
    append_dependency_incidences(&mut incidences, input, space_id);
    append_evidence_incidences(&mut incidences, input, space_id)?;
    Ok(incidences)
}

fn append_file_incidences(
    incidences: &mut Vec<PrReviewTargetLiftedIncidence>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
) -> RuntimeResult<()> {
    for (file_index, file) in input.changed_files.iter().enumerate() {
        for symbol_id in &file.symbol_ids {
            incidences.push(lifted_incidence(
                input,
                space_id.clone(),
                LiftedIncidenceSpec::new(
                    incidence_id("contains", &file.id, symbol_id)?,
                    file.id.clone(),
                    symbol_id.clone(),
                    "contains_symbol",
                    input.source.confidence,
                )
                .with_source_local_id(format!("changed_files[{file_index}].symbol_ids")),
            ));
        }
        append_file_owner_incidences(incidences, input, space_id, file_index, file)?;
    }
    Ok(())
}

fn append_file_owner_incidences(
    incidences: &mut Vec<PrReviewTargetLiftedIncidence>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    file_index: usize,
    file: &crate::pr_review_reports::PrReviewTargetInputChangedFile,
) -> RuntimeResult<()> {
    for owner_id in &file.owner_ids {
        incidences.push(lifted_incidence(
            input,
            space_id.clone(),
            LiftedIncidenceSpec::new(
                incidence_id("owned", &file.id, owner_id)?,
                file.id.clone(),
                owner_id.clone(),
                "owned_by",
                input.source.confidence,
            )
            .with_source_local_id(format!("changed_files[{file_index}].owner_ids")),
        ));
    }
    Ok(())
}

fn append_symbol_incidences(
    incidences: &mut Vec<PrReviewTargetLiftedIncidence>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
) -> RuntimeResult<()> {
    for (symbol_index, symbol) in input.symbols.iter().enumerate() {
        for owner_id in &symbol.owner_ids {
            incidences.push(lifted_incidence(
                input,
                space_id.clone(),
                LiftedIncidenceSpec::new(
                    incidence_id("owned", &symbol.id, owner_id)?,
                    symbol.id.clone(),
                    owner_id.clone(),
                    "owned_by",
                    input.source.confidence,
                )
                .with_source_local_id(format!("symbols[{symbol_index}].owner_ids")),
            ));
        }
    }
    Ok(())
}

fn append_test_incidences(
    incidences: &mut Vec<PrReviewTargetLiftedIncidence>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
) -> RuntimeResult<()> {
    for (test_index, test) in input.tests.iter().enumerate() {
        if let Some(file_id) = &test.file_id {
            incidences.push(test_incidence(
                input, space_id, file_id, &test.id, test_index, "file_id",
            )?);
        }
        for symbol_id in &test.symbol_ids {
            incidences.push(test_incidence(
                input,
                space_id,
                symbol_id,
                &test.id,
                test_index,
                "symbol_ids",
            )?);
        }
    }
    Ok(())
}

fn test_incidence(
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    covered_id: &Id,
    test_id: &Id,
    test_index: usize,
    source_field: &str,
) -> RuntimeResult<PrReviewTargetLiftedIncidence> {
    let source_local_id = format!("tests[{test_index}].{source_field}");
    Ok(lifted_incidence(
        input,
        space_id.clone(),
        LiftedIncidenceSpec::new(
            incidence_id("covered", covered_id, test_id)?,
            covered_id.clone(),
            test_id.clone(),
            "covered_by_test",
            input.source.confidence,
        )
        .with_source_local_id(source_local_id),
    ))
}

fn append_dependency_incidences(
    incidences: &mut Vec<PrReviewTargetLiftedIncidence>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
) {
    incidences.extend(
        input
            .dependency_edges
            .iter()
            .enumerate()
            .map(|(index, edge)| {
                lifted_incidence(
                    input,
                    space_id.clone(),
                    LiftedIncidenceSpec::new(
                        edge.id.clone(),
                        edge.from_id.clone(),
                        edge.to_id.clone(),
                        serde_plain_dependency_relation(edge.relation_type),
                        edge.confidence.unwrap_or(input.source.confidence),
                    )
                    .with_orientation(edge.orientation.unwrap_or(IncidenceOrientation::Directed))
                    .with_source_local_id(format!("dependency_edges[{index}]")),
                )
            }),
    );
}

fn append_evidence_incidences(
    incidences: &mut Vec<PrReviewTargetLiftedIncidence>,
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
) -> RuntimeResult<()> {
    for (index, evidence) in input.evidence.iter().enumerate() {
        for source_id in &evidence.source_ids {
            if cell_id_exists(input, source_id) {
                incidences.push(evidence_incidence(
                    input, space_id, index, evidence, source_id,
                )?);
            }
        }
    }
    Ok(())
}

fn evidence_incidence(
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    index: usize,
    evidence: &crate::pr_review_reports::PrReviewTargetInputEvidence,
    source_id: &Id,
) -> RuntimeResult<PrReviewTargetLiftedIncidence> {
    Ok(lifted_incidence(
        input,
        space_id.clone(),
        LiftedIncidenceSpec::new(
            incidence_id("supports", &evidence.id, source_id)?,
            evidence.id.clone(),
            source_id.clone(),
            "supports",
            evidence.confidence.unwrap_or(input.source.confidence),
        )
        .with_source_local_id(format!("evidence[{index}].source_ids")),
    ))
}

struct LiftedIncidenceSpec {
    id: Id,
    from_cell_id: Id,
    to_cell_id: Id,
    relation_type: String,
    orientation: IncidenceOrientation,
    confidence: Confidence,
    source_local_id: Option<String>,
}

impl LiftedIncidenceSpec {
    fn new(
        id: Id,
        from_cell_id: Id,
        to_cell_id: Id,
        relation_type: impl Into<String>,
        confidence: Confidence,
    ) -> Self {
        Self {
            id,
            from_cell_id,
            to_cell_id,
            relation_type: relation_type.into(),
            orientation: IncidenceOrientation::Directed,
            confidence,
            source_local_id: None,
        }
    }

    fn with_orientation(mut self, orientation: IncidenceOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    fn with_source_local_id(mut self, source_local_id: impl Into<String>) -> Self {
        self.source_local_id = Some(source_local_id.into());
        self
    }
}

fn lifted_incidence(
    input: &PrReviewTargetInputDocument,
    space_id: Id,
    spec: LiftedIncidenceSpec,
) -> PrReviewTargetLiftedIncidence {
    PrReviewTargetLiftedIncidence {
        id: spec.id,
        space_id,
        from_cell_id: spec.from_cell_id,
        to_cell_id: spec.to_cell_id,
        relation_type: spec.relation_type,
        orientation: spec.orientation,
        weight: None,
        provenance: fact_provenance(input, spec.confidence, spec.source_local_id.as_deref()),
    }
}

fn fact_provenance(
    input: &PrReviewTargetInputDocument,
    confidence: Confidence,
    source_local_id: Option<&str>,
) -> Provenance {
    let mut provenance = Provenance::new(source_ref(input, source_local_id), confidence)
        .with_review_status(ReviewStatus::Accepted);
    provenance.extraction_method = Some(EXTRACTION_METHOD.to_owned());
    provenance
}

fn source_ref(input: &PrReviewTargetInputDocument, source_local_id: Option<&str>) -> SourceRef {
    SourceRef {
        kind: input.source.kind.clone(),
        uri: input.source.uri.clone(),
        title: input.source.title.clone(),
        captured_at: input.source.captured_at.clone(),
        source_local_id: source_local_id.map(ToOwned::to_owned),
    }
}

fn effective_context_ids(input: &PrReviewTargetInputDocument) -> RuntimeResult<Vec<Id>> {
    let mut ids = input
        .contexts
        .iter()
        .map(|context| context.id.clone())
        .collect::<Vec<_>>();
    if ids.is_empty() {
        ids.push(id(format!(
            "context:pr-review-target:{}",
            input.pull_request.id
        ))?);
    }
    Ok(ids)
}

fn symbol_context_ids(
    input: &PrReviewTargetInputDocument,
    symbol: &PrReviewTargetInputSymbol,
) -> Vec<Id> {
    if !symbol.context_ids.is_empty() {
        return symbol.context_ids.clone();
    }
    input
        .changed_files
        .iter()
        .find(|file| file.id == symbol.file_id)
        .map(|file| file.context_ids.clone())
        .unwrap_or_default()
}

fn owner_context_ids(
    input: &PrReviewTargetInputDocument,
    owner: &PrReviewTargetInputOwner,
) -> Vec<Id> {
    let mut ids = Vec::new();
    for file in &input.changed_files {
        if file.owner_ids.contains(&owner.id) {
            for context_id in &file.context_ids {
                push_unique(&mut ids, context_id.clone());
            }
        }
    }
    for symbol in &input.symbols {
        if symbol.owner_ids.contains(&owner.id) {
            for context_id in &symbol.context_ids {
                push_unique(&mut ids, context_id.clone());
            }
        }
    }
    ids
}

fn test_context_ids(
    input: &PrReviewTargetInputDocument,
    test: &PrReviewTargetInputTest,
) -> Vec<Id> {
    if !test.context_ids.is_empty() {
        return test.context_ids.clone();
    }
    let mut ids = Vec::new();
    if let Some(file_id) = &test.file_id {
        if let Some(file) = input.changed_files.iter().find(|file| &file.id == file_id) {
            for context_id in &file.context_ids {
                push_unique(&mut ids, context_id.clone());
            }
        }
    }
    for symbol_id in &test.symbol_ids {
        if let Some(symbol) = input.symbols.iter().find(|symbol| &symbol.id == symbol_id) {
            for context_id in &symbol.context_ids {
                push_unique(&mut ids, context_id.clone());
            }
        }
    }
    ids
}

fn contexts_or_default(context_ids: &[Id], default_context_ids: &[Id]) -> Vec<Id> {
    if context_ids.is_empty() {
        return default_context_ids.to_vec();
    }
    context_ids.to_vec()
}

fn file_label(path: &str) -> String {
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_owned()
}

fn cell_id_exists(input: &PrReviewTargetInputDocument, id: &Id) -> bool {
    input.changed_files.iter().any(|file| &file.id == id)
        || input.symbols.iter().any(|symbol| &symbol.id == id)
        || input.owners.iter().any(|owner| &owner.id == id)
        || input.tests.iter().any(|test| &test.id == id)
        || input.evidence.iter().any(|evidence| &evidence.id == id)
        || input.signals.iter().any(|signal| &signal.id == id)
}

fn incidence_id(prefix: &str, from: &Id, to: &Id) -> RuntimeResult<Id> {
    id(format!("incidence:{prefix}:{}:{}", slug(from), slug(to)))
}

fn serde_plain_context_type(
    context_type: crate::pr_review_reports::PrReviewTargetContextType,
) -> String {
    use crate::pr_review_reports::PrReviewTargetContextType;
    match context_type {
        PrReviewTargetContextType::Repository => "repository",
        PrReviewTargetContextType::PullRequest => "pull_request",
        PrReviewTargetContextType::ReviewFocus => "review_focus",
        PrReviewTargetContextType::Ownership => "ownership",
        PrReviewTargetContextType::TestScope => "test_scope",
        PrReviewTargetContextType::DependencyScope => "dependency_scope",
        PrReviewTargetContextType::Custom => "custom",
    }
    .to_owned()
}

fn serde_plain_dependency_relation(
    relation_type: crate::pr_review_reports::PrReviewTargetDependencyRelationType,
) -> String {
    use crate::pr_review_reports::PrReviewTargetDependencyRelationType;
    match relation_type {
        PrReviewTargetDependencyRelationType::Imports => "imports",
        PrReviewTargetDependencyRelationType::Calls => "calls",
        PrReviewTargetDependencyRelationType::Owns => "owns",
        PrReviewTargetDependencyRelationType::Tests => "tests",
        PrReviewTargetDependencyRelationType::Covers => "covers",
        PrReviewTargetDependencyRelationType::DependsOn => "depends_on",
        PrReviewTargetDependencyRelationType::GeneratedFrom => "generated_from",
        PrReviewTargetDependencyRelationType::Custom => "custom",
    }
    .to_owned()
}
