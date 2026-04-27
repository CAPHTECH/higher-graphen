//! Bounded PR review target recommender workflow.

use crate::error::{RuntimeError, RuntimeResult};
use crate::pr_review_reports::{
    PrReviewTarget, PrReviewTargetChangedFile, PrReviewTargetContext, PrReviewTargetDependencyEdge,
    PrReviewTargetEvidence, PrReviewTargetInputChangedFile, PrReviewTargetInputDependencyEdge,
    PrReviewTargetInputDocument, PrReviewTargetInputOwner, PrReviewTargetInputRiskSignal,
    PrReviewTargetInputSymbol, PrReviewTargetInputTest, PrReviewTargetLiftedCell,
    PrReviewTargetLiftedContext, PrReviewTargetLiftedIncidence, PrReviewTargetLiftedSpace,
    PrReviewTargetLiftedStructure, PrReviewTargetLocation, PrReviewTargetObstruction,
    PrReviewTargetObstructionType, PrReviewTargetOwner, PrReviewTargetReport, PrReviewTargetResult,
    PrReviewTargetRiskSignal, PrReviewTargetScenario, PrReviewTargetStatus, PrReviewTargetSymbol,
    PrReviewTargetTest, PrReviewTargetType,
};
use crate::reports::{
    AiProjectionRecord, AiProjectionRecordType, AiProjectionView, AuditProjectionView,
    HumanReviewProjectionView, ProjectionAudience, ProjectionPurpose, ProjectionTrace,
    ProjectionViewSet, ReportEnvelope, ReportMetadata,
};
use higher_graphen_completion::{CompletionCandidate, MissingType, SuggestedStructure};
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, Severity, SourceRef};
use higher_graphen_projection::InformationLoss;
use higher_graphen_space::IncidenceOrientation;

const WORKFLOW_NAME: &str = "pr_review_target";
const INPUT_SCHEMA: &str = "highergraphen.pr_review_target.input.v1";
const REPORT_SCHEMA: &str = "highergraphen.pr_review_target.report.v1";
const REPORT_TYPE: &str = "pr_review_target";
const REPORT_VERSION: u32 = 1;
const EXTRACTION_METHOD: &str = "pr_review_target_lift.v1";

/// Runs the bounded PR review target recommender workflow.
pub fn run_pr_review_target_recommend(
    input: PrReviewTargetInputDocument,
) -> RuntimeResult<PrReviewTargetReport> {
    validate_input_schema(&input)?;
    validate_input_references(&input)?;

    let lifted = lift_input(&input)?;
    let accepted_change_ids = accepted_change_ids(&input);
    let review_targets = recommend_review_targets(&input)?;
    let obstructions = review_obstructions(&input, &review_targets)?;
    let completion_candidates = completion_candidates(&input, &obstructions)?;
    ensure_ai_proposals_are_unreviewed(&review_targets, &obstructions, &completion_candidates)?;

    let mut source_ids = result_source_ids(
        &accepted_change_ids,
        &review_targets,
        &obstructions,
        &completion_candidates,
    );
    if source_ids.is_empty() {
        source_ids = accepted_change_ids.clone();
    }

    let status = if review_targets.is_empty() && obstructions.is_empty() {
        PrReviewTargetStatus::NoTargets
    } else {
        PrReviewTargetStatus::TargetsRecommended
    };
    let result = PrReviewTargetResult {
        status,
        accepted_change_ids,
        review_targets,
        obstructions,
        completion_candidates,
        source_ids,
    };
    let scenario = report_scenario(&input, lifted);
    let projection = report_projection(&input, &scenario, &result)?;

    Ok(ReportEnvelope {
        schema: REPORT_SCHEMA.to_owned(),
        report_type: REPORT_TYPE.to_owned(),
        report_version: REPORT_VERSION,
        metadata: ReportMetadata::pr_review_target(),
        scenario,
        result,
        projection,
    })
}

struct LiftedPrReviewTarget {
    structure: PrReviewTargetLiftedStructure,
}

fn lift_input(input: &PrReviewTargetInputDocument) -> RuntimeResult<LiftedPrReviewTarget> {
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
        .map(|(index, context_id)| {
            if let Some(context) = input
                .contexts
                .iter()
                .find(|context| &context.id == context_id)
            {
                PrReviewTargetLiftedContext {
                    id: context.id.clone(),
                    space_id: space_id.clone(),
                    name: context.name.clone(),
                    context_type: serde_plain_context_type(context.context_type),
                    provenance: fact_provenance(
                        input,
                        input.source.confidence,
                        Some(&format!("contexts[{index}]")),
                    ),
                }
            } else {
                PrReviewTargetLiftedContext {
                    id: context_id.clone(),
                    space_id: space_id.clone(),
                    name: format!("PR {}", input.pull_request.number),
                    context_type: "pull_request".to_owned(),
                    provenance: fact_provenance(
                        input,
                        input.source.confidence,
                        Some("pull_request"),
                    ),
                }
            }
        })
        .collect()
}

fn lifted_cells(
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
) -> RuntimeResult<Vec<PrReviewTargetLiftedCell>> {
    let mut cells = Vec::new();

    for (index, file) in input.changed_files.iter().enumerate() {
        cells.push(PrReviewTargetLiftedCell {
            id: file.id.clone(),
            space_id: space_id.clone(),
            dimension: 0,
            cell_type: "pr.changed_file".to_owned(),
            label: file_label(&file.path),
            context_ids: contexts_or_default(&file.context_ids, default_context_ids),
            provenance: fact_provenance(
                input,
                input.source.confidence,
                Some(&format!("changed_files[{index}]")),
            ),
        });
    }
    for (index, symbol) in input.symbols.iter().enumerate() {
        cells.push(PrReviewTargetLiftedCell {
            id: symbol.id.clone(),
            space_id: space_id.clone(),
            dimension: 0,
            cell_type: "pr.symbol".to_owned(),
            label: symbol.name.clone(),
            context_ids: contexts_or_default(
                &symbol_context_ids(input, symbol),
                default_context_ids,
            ),
            provenance: fact_provenance(
                input,
                input.source.confidence,
                Some(&format!("symbols[{index}]")),
            ),
        });
    }
    for (index, owner) in input.owners.iter().enumerate() {
        cells.push(PrReviewTargetLiftedCell {
            id: owner.id.clone(),
            space_id: space_id.clone(),
            dimension: 0,
            cell_type: "pr.owner".to_owned(),
            label: owner.name.clone().unwrap_or_else(|| owner.id.to_string()),
            context_ids: contexts_or_default(&owner_context_ids(input, owner), default_context_ids),
            provenance: fact_provenance(
                input,
                input.source.confidence,
                Some(&format!("owners[{index}]")),
            ),
        });
    }
    for (index, test) in input.tests.iter().enumerate() {
        cells.push(PrReviewTargetLiftedCell {
            id: test.id.clone(),
            space_id: space_id.clone(),
            dimension: 0,
            cell_type: "pr.test".to_owned(),
            label: test.name.clone(),
            context_ids: contexts_or_default(&test_context_ids(input, test), default_context_ids),
            provenance: fact_provenance(
                input,
                input.source.confidence,
                Some(&format!("tests[{index}]")),
            ),
        });
    }
    for (index, evidence) in input.evidence.iter().enumerate() {
        cells.push(PrReviewTargetLiftedCell {
            id: evidence.id.clone(),
            space_id: space_id.clone(),
            dimension: 0,
            cell_type: "pr.evidence".to_owned(),
            label: evidence.summary.clone(),
            context_ids: contexts_or_default(default_context_ids, default_context_ids),
            provenance: fact_provenance(
                input,
                evidence.confidence.unwrap_or(input.source.confidence),
                Some(&format!("evidence[{index}]")),
            ),
        });
    }
    for (index, signal) in input.signals.iter().enumerate() {
        cells.push(PrReviewTargetLiftedCell {
            id: signal.id.clone(),
            space_id: space_id.clone(),
            dimension: 0,
            cell_type: "pr.risk_signal".to_owned(),
            label: signal.summary.clone(),
            context_ids: contexts_or_default(default_context_ids, default_context_ids),
            provenance: fact_provenance(
                input,
                signal.confidence,
                Some(&format!("signals[{index}]")),
            ),
        });
    }

    Ok(cells)
}

fn lifted_incidences(
    input: &PrReviewTargetInputDocument,
    space_id: &Id,
) -> RuntimeResult<Vec<PrReviewTargetLiftedIncidence>> {
    let mut incidences = Vec::new();

    for (file_index, file) in input.changed_files.iter().enumerate() {
        for symbol_id in &file.symbol_ids {
            incidences.push(lifted_incidence(
                input,
                incidence_id("contains", &file.id, symbol_id)?,
                space_id.clone(),
                file.id.clone(),
                symbol_id.clone(),
                "contains_symbol",
                IncidenceOrientation::Directed,
                input.source.confidence,
                Some(&format!("changed_files[{file_index}].symbol_ids")),
            ));
        }
        for owner_id in &file.owner_ids {
            incidences.push(lifted_incidence(
                input,
                incidence_id("owned", &file.id, owner_id)?,
                space_id.clone(),
                file.id.clone(),
                owner_id.clone(),
                "owned_by",
                IncidenceOrientation::Directed,
                input.source.confidence,
                Some(&format!("changed_files[{file_index}].owner_ids")),
            ));
        }
    }
    for (symbol_index, symbol) in input.symbols.iter().enumerate() {
        for owner_id in &symbol.owner_ids {
            incidences.push(lifted_incidence(
                input,
                incidence_id("owned", &symbol.id, owner_id)?,
                space_id.clone(),
                symbol.id.clone(),
                owner_id.clone(),
                "owned_by",
                IncidenceOrientation::Directed,
                input.source.confidence,
                Some(&format!("symbols[{symbol_index}].owner_ids")),
            ));
        }
    }
    for (test_index, test) in input.tests.iter().enumerate() {
        if let Some(file_id) = &test.file_id {
            incidences.push(lifted_incidence(
                input,
                incidence_id("covered", file_id, &test.id)?,
                space_id.clone(),
                file_id.clone(),
                test.id.clone(),
                "covered_by_test",
                IncidenceOrientation::Directed,
                input.source.confidence,
                Some(&format!("tests[{test_index}].file_id")),
            ));
        }
        for symbol_id in &test.symbol_ids {
            incidences.push(lifted_incidence(
                input,
                incidence_id("covered", symbol_id, &test.id)?,
                space_id.clone(),
                symbol_id.clone(),
                test.id.clone(),
                "covered_by_test",
                IncidenceOrientation::Directed,
                input.source.confidence,
                Some(&format!("tests[{test_index}].symbol_ids")),
            ));
        }
    }
    for (index, edge) in input.dependency_edges.iter().enumerate() {
        incidences.push(lifted_incidence(
            input,
            edge.id.clone(),
            space_id.clone(),
            edge.from_id.clone(),
            edge.to_id.clone(),
            serde_plain_dependency_relation(edge.relation_type),
            edge.orientation.unwrap_or(IncidenceOrientation::Directed),
            edge.confidence.unwrap_or(input.source.confidence),
            Some(&format!("dependency_edges[{index}]")),
        ));
    }
    for (index, evidence) in input.evidence.iter().enumerate() {
        for source_id in &evidence.source_ids {
            if cell_id_exists(input, source_id) {
                incidences.push(lifted_incidence(
                    input,
                    incidence_id("supports", &evidence.id, source_id)?,
                    space_id.clone(),
                    evidence.id.clone(),
                    source_id.clone(),
                    "supports",
                    IncidenceOrientation::Directed,
                    evidence.confidence.unwrap_or(input.source.confidence),
                    Some(&format!("evidence[{index}].source_ids")),
                ));
            }
        }
    }

    Ok(incidences)
}

fn lifted_incidence(
    input: &PrReviewTargetInputDocument,
    id: Id,
    space_id: Id,
    from_cell_id: Id,
    to_cell_id: Id,
    relation_type: impl Into<String>,
    orientation: IncidenceOrientation,
    confidence: Confidence,
    source_local_id: Option<&str>,
) -> PrReviewTargetLiftedIncidence {
    PrReviewTargetLiftedIncidence {
        id,
        space_id,
        from_cell_id,
        to_cell_id,
        relation_type: relation_type.into(),
        orientation,
        weight: None,
        provenance: fact_provenance(input, confidence, source_local_id),
    }
}

fn report_scenario(
    input: &PrReviewTargetInputDocument,
    lifted: LiftedPrReviewTarget,
) -> PrReviewTargetScenario {
    PrReviewTargetScenario {
        input_schema: input.schema.clone(),
        source: input.source.clone(),
        repository: input.repository.clone(),
        pull_request: input.pull_request.clone(),
        changed_files: input
            .changed_files
            .iter()
            .map(|file| PrReviewTargetChangedFile {
                id: file.id.clone(),
                path: file.path.clone(),
                change_type: file.change_type,
                language: file.language.clone(),
                additions: file.additions,
                deletions: file.deletions,
                symbol_ids: file.symbol_ids.clone(),
                owner_ids: file.owner_ids.clone(),
                context_ids: file.context_ids.clone(),
                review_status: ReviewStatus::Accepted,
                confidence: input.source.confidence,
            })
            .collect(),
        symbols: input
            .symbols
            .iter()
            .map(|symbol| PrReviewTargetSymbol {
                id: symbol.id.clone(),
                file_id: symbol.file_id.clone(),
                name: symbol.name.clone(),
                kind: symbol.kind,
                path: symbol.path.clone(),
                line_start: symbol.line_start,
                line_end: symbol.line_end,
                owner_ids: symbol.owner_ids.clone(),
                context_ids: symbol.context_ids.clone(),
                review_status: ReviewStatus::Accepted,
                confidence: input.source.confidence,
            })
            .collect(),
        owners: input
            .owners
            .iter()
            .map(|owner| PrReviewTargetOwner {
                id: owner.id.clone(),
                owner_type: owner.owner_type,
                name: owner.name.clone(),
                source_ids: owner.source_ids.clone(),
                review_status: ReviewStatus::Accepted,
                confidence: input.source.confidence,
            })
            .collect(),
        contexts: input
            .contexts
            .iter()
            .map(|context| PrReviewTargetContext {
                id: context.id.clone(),
                name: context.name.clone(),
                context_type: context.context_type,
                source_ids: context.source_ids.clone(),
                review_status: ReviewStatus::Accepted,
                confidence: input.source.confidence,
            })
            .collect(),
        tests: input
            .tests
            .iter()
            .map(|test| PrReviewTargetTest {
                id: test.id.clone(),
                name: test.name.clone(),
                test_type: test.test_type,
                file_id: test.file_id.clone(),
                symbol_ids: test.symbol_ids.clone(),
                context_ids: test.context_ids.clone(),
                source_ids: test.source_ids.clone(),
                review_status: ReviewStatus::Accepted,
                confidence: input.source.confidence,
            })
            .collect(),
        dependency_edges: input
            .dependency_edges
            .iter()
            .map(|edge| PrReviewTargetDependencyEdge {
                id: edge.id.clone(),
                from_id: edge.from_id.clone(),
                to_id: edge.to_id.clone(),
                relation_type: edge.relation_type,
                orientation: edge.orientation,
                source_ids: edge.source_ids.clone(),
                review_status: ReviewStatus::Accepted,
                confidence: edge.confidence.unwrap_or(input.source.confidence),
            })
            .collect(),
        evidence: input
            .evidence
            .iter()
            .map(|evidence| PrReviewTargetEvidence {
                id: evidence.id.clone(),
                evidence_type: evidence.evidence_type,
                summary: evidence.summary.clone(),
                source_ids: evidence.source_ids.clone(),
                review_status: ReviewStatus::Accepted,
                confidence: evidence.confidence.unwrap_or(input.source.confidence),
            })
            .collect(),
        signals: input
            .signals
            .iter()
            .map(|signal| PrReviewTargetRiskSignal {
                id: signal.id.clone(),
                signal_type: signal.signal_type,
                summary: signal.summary.clone(),
                source_ids: signal.source_ids.clone(),
                severity: signal.severity,
                confidence: signal.confidence,
            })
            .collect(),
        reviewer_context: input.reviewer_context.clone(),
        lifted_structure: lifted.structure,
    }
}

fn recommend_review_targets(
    input: &PrReviewTargetInputDocument,
) -> RuntimeResult<Vec<PrReviewTarget>> {
    let mut targets = Vec::new();
    for signal in &input.signals {
        let mut signal_targets = Vec::new();
        for source_id in &signal.source_ids {
            if let Some(target) = target_for_signal_source(input, signal, source_id)? {
                signal_targets.push(target);
            }
        }
        if signal_targets.is_empty() && !signal.source_ids.is_empty() {
            signal_targets.push(cross_cutting_target(input, signal)?);
        }
        for target in signal_targets {
            push_unique_target(&mut targets, target);
        }
    }
    Ok(targets)
}

fn target_for_signal_source(
    input: &PrReviewTargetInputDocument,
    signal: &PrReviewTargetInputRiskSignal,
    source_id: &Id,
) -> RuntimeResult<Option<PrReviewTarget>> {
    if let Some(edge) = input
        .dependency_edges
        .iter()
        .find(|edge| &edge.id == source_id)
    {
        return dependency_target(input, signal, edge).map(Some);
    }
    if let Some(symbol) = input.symbols.iter().find(|symbol| &symbol.id == source_id) {
        if symbol_file(input, symbol).is_some_and(|file| is_excluded(input, &file.path)) {
            return Ok(None);
        }
        return symbol_target(input, signal, symbol).map(Some);
    }
    if let Some(file) = input
        .changed_files
        .iter()
        .find(|file| &file.id == source_id)
    {
        if is_excluded(input, &file.path) {
            return Ok(None);
        }
        if let Some(symbol) = file
            .symbol_ids
            .iter()
            .find_map(|symbol_id| input.symbols.iter().find(|symbol| &symbol.id == symbol_id))
        {
            return symbol_target(input, signal, symbol).map(Some);
        }
        return file_target(input, signal, file).map(Some);
    }
    if let Some(test) = input.tests.iter().find(|test| &test.id == source_id) {
        return test_target(input, signal, test).map(Some);
    }
    Ok(None)
}

fn symbol_target(
    input: &PrReviewTargetInputDocument,
    signal: &PrReviewTargetInputRiskSignal,
    symbol: &PrReviewTargetInputSymbol,
) -> RuntimeResult<PrReviewTarget> {
    let file = symbol_file(input, symbol);
    let path = symbol
        .path
        .clone()
        .or_else(|| file.map(|file| file.path.clone()))
        .unwrap_or_else(|| symbol.name.clone());
    let id = target_id(signal, &symbol.id)?;
    Ok(PrReviewTarget {
        id,
        target_type: PrReviewTargetType::Symbol,
        target_ref: symbol.id.to_string(),
        title: format!("Review {}", symbol.name),
        rationale: format!(
            "{} This symbol is in the bounded PR snapshot and is linked to the risk signal.",
            signal.summary
        ),
        evidence_ids: evidence_ids(signal, Some(&symbol.id)),
        location: Some(PrReviewTargetLocation {
            path,
            line_start: symbol.line_start,
            line_end: symbol.line_end,
            symbol_id: Some(symbol.id.clone()),
        }),
        suggested_questions: questions_for_signal(signal),
        related_target_ids: Vec::new(),
        severity: recommended_severity(signal),
        confidence: signal.confidence,
        review_status: ReviewStatus::Unreviewed,
    })
}

fn file_target(
    _input: &PrReviewTargetInputDocument,
    signal: &PrReviewTargetInputRiskSignal,
    file: &PrReviewTargetInputChangedFile,
) -> RuntimeResult<PrReviewTarget> {
    Ok(PrReviewTarget {
        id: target_id(signal, &file.id)?,
        target_type: PrReviewTargetType::File,
        target_ref: file.path.clone(),
        title: format!("Review {}", file_label(&file.path)),
        rationale: format!(
            "{} This changed file is directly referenced by the risk signal.",
            signal.summary
        ),
        evidence_ids: evidence_ids(signal, Some(&file.id)),
        location: Some(PrReviewTargetLocation {
            path: file.path.clone(),
            line_start: None,
            line_end: None,
            symbol_id: None,
        }),
        suggested_questions: questions_for_signal(signal),
        related_target_ids: Vec::new(),
        severity: recommended_severity(signal),
        confidence: signal.confidence,
        review_status: ReviewStatus::Unreviewed,
    })
}

fn dependency_target(
    _input: &PrReviewTargetInputDocument,
    signal: &PrReviewTargetInputRiskSignal,
    edge: &PrReviewTargetInputDependencyEdge,
) -> RuntimeResult<PrReviewTarget> {
    Ok(PrReviewTarget {
        id: target_id(signal, &edge.id)?,
        target_type: PrReviewTargetType::Dependency,
        target_ref: edge.id.to_string(),
        title: format!("Review dependency {}", edge.id),
        rationale: format!(
            "{} This dependency edge may affect review scope.",
            signal.summary
        ),
        evidence_ids: evidence_ids(signal, Some(&edge.id)),
        location: None,
        suggested_questions: questions_for_signal(signal),
        related_target_ids: Vec::new(),
        severity: recommended_severity(signal),
        confidence: signal.confidence,
        review_status: ReviewStatus::Unreviewed,
    })
}

fn test_target(
    _input: &PrReviewTargetInputDocument,
    signal: &PrReviewTargetInputRiskSignal,
    test: &PrReviewTargetInputTest,
) -> RuntimeResult<PrReviewTarget> {
    Ok(PrReviewTarget {
        id: target_id(signal, &test.id)?,
        target_type: PrReviewTargetType::Test,
        target_ref: test.id.to_string(),
        title: format!("Review test {}", test.name),
        rationale: format!(
            "{} This test is referenced by the risk signal.",
            signal.summary
        ),
        evidence_ids: evidence_ids(signal, Some(&test.id)),
        location: None,
        suggested_questions: questions_for_signal(signal),
        related_target_ids: Vec::new(),
        severity: recommended_severity(signal),
        confidence: signal.confidence,
        review_status: ReviewStatus::Unreviewed,
    })
}

fn cross_cutting_target(
    _input: &PrReviewTargetInputDocument,
    signal: &PrReviewTargetInputRiskSignal,
) -> RuntimeResult<PrReviewTarget> {
    Ok(PrReviewTarget {
        id: id(format!("target:{}", slug(&signal.id)))?,
        target_type: PrReviewTargetType::CrossCutting,
        target_ref: signal.id.to_string(),
        title: "Review cross-cutting PR risk".to_owned(),
        rationale: signal.summary.clone(),
        evidence_ids: evidence_ids(signal, None),
        location: None,
        suggested_questions: questions_for_signal(signal),
        related_target_ids: Vec::new(),
        severity: recommended_severity(signal),
        confidence: signal.confidence,
        review_status: ReviewStatus::Unreviewed,
    })
}

fn review_obstructions(
    input: &PrReviewTargetInputDocument,
    review_targets: &[PrReviewTarget],
) -> RuntimeResult<Vec<PrReviewTargetObstruction>> {
    let mut obstructions = Vec::new();
    for signal in &input.signals {
        let Some(obstruction_type) = obstruction_type_for_signal(signal) else {
            continue;
        };
        let has_target = review_targets
            .iter()
            .any(|target| target.evidence_ids.contains(&signal.id));
        if !has_target {
            continue;
        }
        obstructions.push(PrReviewTargetObstruction {
            id: id(format!("obstruction:{}", slug(&signal.id)))?,
            obstruction_type,
            summary: signal.summary.clone(),
            required_resolution: Some(
                "A human reviewer should inspect the related unreviewed targets before treating this PR as reviewed."
                    .to_owned(),
            ),
            severity: recommended_severity(signal),
            source_ids: evidence_ids(signal, None),
            confidence: signal.confidence,
            review_status: ReviewStatus::Unreviewed,
        });
    }
    Ok(obstructions)
}

fn completion_candidates(
    input: &PrReviewTargetInputDocument,
    obstructions: &[PrReviewTargetObstruction],
) -> RuntimeResult<Vec<CompletionCandidate>> {
    if obstructions.is_empty() {
        return Ok(Vec::new());
    }
    let space_id = space_id(input)?;
    let candidate_id = id(format!(
        "candidate:{}:review-checklist",
        slug(&input.pull_request.id)
    ))?;
    let structure_id = id(format!(
        "section:{}:review-checklist",
        slug(&input.pull_request.id)
    ))?;
    let suggested = SuggestedStructure::new(
        "review_checklist",
        "Add a checklist section for the unreviewed PR review targets.",
    )?
    .with_structure_id(structure_id)
    .with_related_ids(
        obstructions
            .iter()
            .map(|obstruction| obstruction.id.clone())
            .collect(),
    );
    let candidate = CompletionCandidate::new(
        candidate_id,
        space_id,
        MissingType::Section,
        suggested,
        obstructions
            .iter()
            .map(|obstruction| obstruction.id.clone())
            .collect(),
        "The recommender found unresolved review risks that should be tracked explicitly.",
        Confidence::new(0.71)?,
    )?;
    Ok(vec![candidate])
}

fn report_projection(
    input: &PrReviewTargetInputDocument,
    scenario: &PrReviewTargetScenario,
    result: &PrReviewTargetResult,
) -> RuntimeResult<ProjectionViewSet> {
    let source_ids = if result.source_ids.is_empty() {
        result.accepted_change_ids.clone()
    } else {
        result.source_ids.clone()
    };
    let human_loss = InformationLoss::declared(
        "Projection summarizes changed files, symbols, risk signals, targets, obstructions, and completion candidates without embedding raw provider payloads.",
        source_ids.clone(),
    )?;
    let ai_loss = InformationLoss::declared(
        "AI view preserves stable IDs, severity, confidence, and review status but omits full changed-file payloads.",
        source_ids.clone(),
    )?;
    let audit_loss = InformationLoss::declared(
        "Audit trace records represented source identifiers and view coverage but omits raw diff hunks and provider payloads.",
        source_ids.clone(),
    )?;
    let human_review = HumanReviewProjectionView {
        audience: ProjectionAudience::Human,
        purpose: ProjectionPurpose::PrReviewTargeting,
        summary: human_summary(result),
        recommended_actions: vec![
            "Review the unreviewed targets before treating this PR as covered.".to_owned(),
            "Record explicit accept or reject decisions outside this recommendation report."
                .to_owned(),
        ],
        source_ids: source_ids.clone(),
        information_loss: vec![human_loss],
    };
    let ai_view = AiProjectionView {
        audience: ProjectionAudience::AiAgent,
        purpose: ProjectionPurpose::PrReviewTargeting,
        records: ai_projection_records(input, scenario, result),
        source_ids: source_ids.clone(),
        information_loss: vec![ai_loss],
    };
    let audit_trace = AuditProjectionView {
        audience: ProjectionAudience::Audit,
        purpose: ProjectionPurpose::AuditTrace,
        source_ids,
        information_loss: vec![audit_loss],
        traces: audit_traces(result.source_ids.clone()),
    };

    Ok(ProjectionViewSet {
        audience: human_review.audience,
        purpose: human_review.purpose,
        summary: human_review.summary.clone(),
        recommended_actions: human_review.recommended_actions.clone(),
        information_loss: human_review.information_loss.clone(),
        source_ids: human_review.source_ids.clone(),
        human_review,
        ai_view,
        audit_trace,
    })
}

fn ai_projection_records(
    input: &PrReviewTargetInputDocument,
    scenario: &PrReviewTargetScenario,
    result: &PrReviewTargetResult,
) -> Vec<AiProjectionRecord> {
    let mut records = Vec::new();
    for file in &scenario.changed_files {
        records.push(AiProjectionRecord {
            id: file.id.clone(),
            record_type: AiProjectionRecordType::ChangedFile,
            summary: format!("{} changed file.", file_label(&file.path)),
            source_ids: nonempty_source_ids(input, &file.id),
            confidence: Some(file.confidence),
            review_status: Some(file.review_status),
            severity: None,
            provenance: None,
        });
    }
    for symbol in &scenario.symbols {
        records.push(AiProjectionRecord {
            id: symbol.id.clone(),
            record_type: AiProjectionRecordType::Symbol,
            summary: format!("Changed symbol {}.", symbol.name),
            source_ids: vec![symbol.id.clone(), symbol.file_id.clone()],
            confidence: Some(symbol.confidence),
            review_status: Some(symbol.review_status),
            severity: None,
            provenance: None,
        });
    }
    for signal in &scenario.signals {
        records.push(AiProjectionRecord {
            id: signal.id.clone(),
            record_type: AiProjectionRecordType::RiskSignal,
            summary: signal.summary.clone(),
            source_ids: signal.source_ids.clone(),
            confidence: Some(signal.confidence),
            review_status: Some(ReviewStatus::Accepted),
            severity: Some(signal.severity),
            provenance: None,
        });
    }
    for target in &result.review_targets {
        records.push(AiProjectionRecord {
            id: target.id.clone(),
            record_type: AiProjectionRecordType::ReviewTarget,
            summary: target.title.clone(),
            source_ids: target_source_ids(target),
            confidence: Some(target.confidence),
            review_status: Some(target.review_status),
            severity: Some(target.severity),
            provenance: None,
        });
    }
    for obstruction in &result.obstructions {
        records.push(AiProjectionRecord {
            id: obstruction.id.clone(),
            record_type: AiProjectionRecordType::Obstruction,
            summary: obstruction.summary.clone(),
            source_ids: obstruction.source_ids.clone(),
            confidence: Some(obstruction.confidence),
            review_status: Some(obstruction.review_status),
            severity: Some(obstruction.severity),
            provenance: None,
        });
    }
    for candidate in &result.completion_candidates {
        records.push(AiProjectionRecord {
            id: candidate.id.clone(),
            record_type: AiProjectionRecordType::CompletionCandidate,
            summary: candidate.suggested_structure.summary.clone(),
            source_ids: completion_candidate_source_ids(candidate),
            confidence: Some(candidate.confidence),
            review_status: Some(candidate.review_status),
            severity: None,
            provenance: None,
        });
    }
    records
}

fn validate_input_schema(input: &PrReviewTargetInputDocument) -> RuntimeResult<()> {
    if input.schema == INPUT_SCHEMA {
        return Ok(());
    }
    Err(RuntimeError::unsupported_input_schema(
        input.schema.clone(),
        INPUT_SCHEMA,
    ))
}

fn validate_input_references(input: &PrReviewTargetInputDocument) -> RuntimeResult<()> {
    if input.changed_files.is_empty() {
        return Err(validation_error(
            "changed_files must contain at least one file",
        ));
    }
    ensure_unique_input_ids(input)?;
    let file_ids = input
        .changed_files
        .iter()
        .map(|file| file.id.clone())
        .collect::<Vec<_>>();
    let symbol_ids = input
        .symbols
        .iter()
        .map(|symbol| symbol.id.clone())
        .collect::<Vec<_>>();
    let owner_ids = input
        .owners
        .iter()
        .map(|owner| owner.id.clone())
        .collect::<Vec<_>>();
    let context_ids = input
        .contexts
        .iter()
        .map(|context| context.id.clone())
        .collect::<Vec<_>>();
    let test_ids = input
        .tests
        .iter()
        .map(|test| test.id.clone())
        .collect::<Vec<_>>();
    let accepted_ids = accepted_input_ids(input);

    for file in &input.changed_files {
        ensure_known_ids(
            &symbol_ids,
            &file.symbol_ids,
            "changed_file",
            &file.id,
            "symbol",
        )?;
        ensure_known_ids(
            &owner_ids,
            &file.owner_ids,
            "changed_file",
            &file.id,
            "owner",
        )?;
        ensure_known_ids(
            &context_ids,
            &file.context_ids,
            "changed_file",
            &file.id,
            "context",
        )?;
        ensure_known_ids(
            &accepted_ids,
            &file.source_ids,
            "changed_file",
            &file.id,
            "source",
        )?;
    }
    for symbol in &input.symbols {
        ensure_known_id(&file_ids, &symbol.file_id, "symbol", &symbol.id, "file")?;
        ensure_known_ids(&owner_ids, &symbol.owner_ids, "symbol", &symbol.id, "owner")?;
        ensure_known_ids(
            &context_ids,
            &symbol.context_ids,
            "symbol",
            &symbol.id,
            "context",
        )?;
    }
    for owner in &input.owners {
        ensure_known_ids(
            &accepted_ids,
            &owner.source_ids,
            "owner",
            &owner.id,
            "source",
        )?;
    }
    for context in &input.contexts {
        ensure_known_ids(
            &accepted_ids,
            &context.source_ids,
            "context",
            &context.id,
            "source",
        )?;
    }
    for test in &input.tests {
        if let Some(file_id) = &test.file_id {
            ensure_known_id(&file_ids, file_id, "test", &test.id, "file")?;
        }
        ensure_known_ids(&symbol_ids, &test.symbol_ids, "test", &test.id, "symbol")?;
        ensure_known_ids(&context_ids, &test.context_ids, "test", &test.id, "context")?;
        ensure_known_ids(&accepted_ids, &test.source_ids, "test", &test.id, "source")?;
    }
    let dependency_endpoint_ids = file_ids
        .iter()
        .chain(symbol_ids.iter())
        .chain(test_ids.iter())
        .chain(owner_ids.iter())
        .cloned()
        .collect::<Vec<_>>();
    for edge in &input.dependency_edges {
        ensure_known_id(
            &dependency_endpoint_ids,
            &edge.from_id,
            "dependency_edge",
            &edge.id,
            "from endpoint",
        )?;
        ensure_known_id(
            &dependency_endpoint_ids,
            &edge.to_id,
            "dependency_edge",
            &edge.id,
            "to endpoint",
        )?;
        ensure_known_ids(
            &accepted_ids,
            &edge.source_ids,
            "dependency_edge",
            &edge.id,
            "source",
        )?;
    }
    for evidence in &input.evidence {
        ensure_known_ids(
            &accepted_ids,
            &evidence.source_ids,
            "evidence",
            &evidence.id,
            "source",
        )?;
    }
    for signal in &input.signals {
        ensure_known_ids(
            &accepted_ids,
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
    for file in &input.changed_files {
        ensure_unique_id(&mut seen, &file.id, "changed_file")?;
    }
    for symbol in &input.symbols {
        ensure_unique_id(&mut seen, &symbol.id, "symbol")?;
    }
    for owner in &input.owners {
        ensure_unique_id(&mut seen, &owner.id, "owner")?;
    }
    for context in &input.contexts {
        ensure_unique_id(&mut seen, &context.id, "context")?;
    }
    for test in &input.tests {
        ensure_unique_id(&mut seen, &test.id, "test")?;
    }
    for edge in &input.dependency_edges {
        ensure_unique_id(&mut seen, &edge.id, "dependency_edge")?;
    }
    for evidence in &input.evidence {
        ensure_unique_id(&mut seen, &evidence.id, "evidence")?;
    }
    for signal in &input.signals {
        ensure_unique_id(&mut seen, &signal.id, "signal")?;
    }
    Ok(())
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

fn accepted_change_ids(input: &PrReviewTargetInputDocument) -> Vec<Id> {
    let mut ids = Vec::new();
    ids.extend(input.changed_files.iter().map(|file| file.id.clone()));
    ids.extend(input.symbols.iter().map(|symbol| symbol.id.clone()));
    ids.extend(input.tests.iter().map(|test| test.id.clone()));
    ids.extend(input.dependency_edges.iter().map(|edge| edge.id.clone()));
    ids
}

fn result_source_ids(
    accepted_change_ids: &[Id],
    review_targets: &[PrReviewTarget],
    obstructions: &[PrReviewTargetObstruction],
    completion_candidates: &[CompletionCandidate],
) -> Vec<Id> {
    let mut ids = Vec::new();
    for id in accepted_change_ids {
        push_unique(&mut ids, id.clone());
    }
    for target in review_targets {
        push_unique(&mut ids, target.id.clone());
        for evidence_id in &target.evidence_ids {
            push_unique(&mut ids, evidence_id.clone());
        }
    }
    for obstruction in obstructions {
        push_unique(&mut ids, obstruction.id.clone());
        for source_id in &obstruction.source_ids {
            push_unique(&mut ids, source_id.clone());
        }
    }
    for candidate in completion_candidates {
        push_unique(&mut ids, candidate.id.clone());
        for source_id in &candidate.inferred_from {
            push_unique(&mut ids, source_id.clone());
        }
    }
    ids
}

fn ensure_ai_proposals_are_unreviewed(
    review_targets: &[PrReviewTarget],
    obstructions: &[PrReviewTargetObstruction],
    completion_candidates: &[CompletionCandidate],
) -> RuntimeResult<()> {
    if review_targets
        .iter()
        .any(|target| target.review_status != ReviewStatus::Unreviewed)
    {
        return Err(validation_error("review targets must remain unreviewed"));
    }
    if obstructions
        .iter()
        .any(|obstruction| obstruction.review_status != ReviewStatus::Unreviewed)
    {
        return Err(validation_error("obstructions must remain unreviewed"));
    }
    if completion_candidates
        .iter()
        .any(|candidate| candidate.review_status != ReviewStatus::Unreviewed)
    {
        return Err(validation_error(
            "completion candidates must remain unreviewed",
        ));
    }
    Ok(())
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

fn space_id(input: &PrReviewTargetInputDocument) -> RuntimeResult<Id> {
    id(format!("space:pr-review-target:{}", input.pull_request.id))
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

fn cell_id_exists(input: &PrReviewTargetInputDocument, id: &Id) -> bool {
    input.changed_files.iter().any(|file| &file.id == id)
        || input.symbols.iter().any(|symbol| &symbol.id == id)
        || input.owners.iter().any(|owner| &owner.id == id)
        || input.tests.iter().any(|test| &test.id == id)
        || input.evidence.iter().any(|evidence| &evidence.id == id)
        || input.signals.iter().any(|signal| &signal.id == id)
}

fn symbol_file<'a>(
    input: &'a PrReviewTargetInputDocument,
    symbol: &PrReviewTargetInputSymbol,
) -> Option<&'a PrReviewTargetInputChangedFile> {
    input
        .changed_files
        .iter()
        .find(|file| file.id == symbol.file_id)
}

fn is_excluded(input: &PrReviewTargetInputDocument, path: &str) -> bool {
    input.reviewer_context.as_ref().is_some_and(|context| {
        context
            .excluded_paths
            .iter()
            .any(|prefix| path.starts_with(prefix))
    })
}

fn recommended_severity(signal: &PrReviewTargetInputRiskSignal) -> Severity {
    if matches!(
        signal.signal_type,
        crate::pr_review_reports::PrReviewTargetRiskSignalType::SecuritySensitive
    ) && signal.severity < Severity::High
    {
        Severity::High
    } else {
        signal.severity
    }
}

fn obstruction_type_for_signal(
    signal: &PrReviewTargetInputRiskSignal,
) -> Option<PrReviewTargetObstructionType> {
    use crate::pr_review_reports::PrReviewTargetRiskSignalType;
    match signal.signal_type {
        PrReviewTargetRiskSignalType::TestGap => Some(PrReviewTargetObstructionType::TestGap),
        PrReviewTargetRiskSignalType::DependencyChange => {
            Some(PrReviewTargetObstructionType::DependencyRisk)
        }
        PrReviewTargetRiskSignalType::OwnershipBoundary => {
            Some(PrReviewTargetObstructionType::OwnershipBoundary)
        }
        PrReviewTargetRiskSignalType::SecuritySensitive => {
            Some(PrReviewTargetObstructionType::SecuritySensitiveChange)
        }
        PrReviewTargetRiskSignalType::LargeChange
        | PrReviewTargetRiskSignalType::GeneratedCode
        | PrReviewTargetRiskSignalType::Custom => None,
    }
}

fn evidence_ids(signal: &PrReviewTargetInputRiskSignal, target_id: Option<&Id>) -> Vec<Id> {
    let mut ids = Vec::new();
    if let Some(target_id) = target_id {
        push_unique(&mut ids, target_id.clone());
    }
    push_unique(&mut ids, signal.id.clone());
    for source_id in &signal.source_ids {
        push_unique(&mut ids, source_id.clone());
    }
    ids
}

fn questions_for_signal(signal: &PrReviewTargetInputRiskSignal) -> Vec<String> {
    use crate::pr_review_reports::PrReviewTargetRiskSignalType;
    match signal.signal_type {
        PrReviewTargetRiskSignalType::TestGap => vec![
            "Does the changed behavior have focused verification coverage?".to_owned(),
            "Are unreviewed recommendations kept separate from accepted review coverage?"
                .to_owned(),
        ],
        PrReviewTargetRiskSignalType::DependencyChange => vec![
            "Does the dependency change alter runtime or review boundaries?".to_owned(),
            "Are downstream callers or tests represented in the bounded snapshot?".to_owned(),
        ],
        PrReviewTargetRiskSignalType::OwnershipBoundary => vec![
            "Should another owner or domain expert inspect this change?".to_owned(),
            "Does the review scope cross an ownership boundary?".to_owned(),
        ],
        PrReviewTargetRiskSignalType::SecuritySensitive => vec![
            "Could this change affect authorization, secrets, or trust boundaries?".to_owned(),
            "Is there explicit evidence for the security-sensitive behavior?".to_owned(),
        ],
        PrReviewTargetRiskSignalType::LargeChange
        | PrReviewTargetRiskSignalType::GeneratedCode
        | PrReviewTargetRiskSignalType::Custom => vec![
            "What human inspection would reduce the highest uncertainty in this change?".to_owned(),
        ],
    }
}

fn target_id(signal: &PrReviewTargetInputRiskSignal, target_id: &Id) -> RuntimeResult<Id> {
    id(format!("target:{}:{}", slug(&signal.id), slug(target_id)))
}

fn incidence_id(prefix: &str, from: &Id, to: &Id) -> RuntimeResult<Id> {
    id(format!("incidence:{prefix}:{}:{}", slug(from), slug(to)))
}

fn slug(id: &Id) -> String {
    id.as_str()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

fn file_label(path: &str) -> String {
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_owned()
}

fn human_summary(result: &PrReviewTargetResult) -> String {
    match result.status {
        PrReviewTargetStatus::TargetsRecommended => format!(
            "Recommended {} unreviewed PR review targets with {} unresolved obstructions.",
            result.review_targets.len(),
            result.obstructions.len()
        ),
        PrReviewTargetStatus::NoTargets => {
            "No PR review targets were recommended from the bounded snapshot.".to_owned()
        }
        PrReviewTargetStatus::UnsupportedInput => {
            "The bounded snapshot could not be mapped into review targets.".to_owned()
        }
    }
}

fn nonempty_source_ids(input: &PrReviewTargetInputDocument, source_id: &Id) -> Vec<Id> {
    let mut ids = vec![source_id.clone()];
    push_unique(&mut ids, input.pull_request.id.clone());
    ids
}

fn target_source_ids(target: &PrReviewTarget) -> Vec<Id> {
    let mut ids = vec![target.id.clone()];
    for evidence_id in &target.evidence_ids {
        push_unique(&mut ids, evidence_id.clone());
    }
    ids
}

fn completion_candidate_source_ids(candidate: &CompletionCandidate) -> Vec<Id> {
    let mut ids = vec![candidate.id.clone(), candidate.space_id.clone()];
    if let Some(structure_id) = &candidate.suggested_structure.structure_id {
        push_unique(&mut ids, structure_id.clone());
    }
    for related_id in &candidate.suggested_structure.related_ids {
        push_unique(&mut ids, related_id.clone());
    }
    for inferred_from in &candidate.inferred_from {
        push_unique(&mut ids, inferred_from.clone());
    }
    ids
}

fn audit_traces(source_ids: Vec<Id>) -> Vec<ProjectionTrace> {
    source_ids
        .into_iter()
        .map(|source_id| ProjectionTrace {
            role: source_role(&source_id).to_owned(),
            source_id,
            represented_in: vec![
                "human_review".to_owned(),
                "ai_view".to_owned(),
                "audit_trace".to_owned(),
            ],
        })
        .collect()
}

fn source_role(source_id: &Id) -> &'static str {
    let value = source_id.as_str();
    if value.starts_with("file:") {
        "changed_file"
    } else if value.starts_with("symbol:") {
        "symbol"
    } else if value.starts_with("signal:") {
        "risk_signal"
    } else if value.starts_with("target:") {
        "review_target"
    } else if value.starts_with("obstruction:") {
        "obstruction"
    } else if value.starts_with("candidate:") {
        "completion_candidate"
    } else if value.starts_with("dependency:") {
        "dependency_edge"
    } else {
        "source"
    }
}

fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn push_unique_target(targets: &mut Vec<PrReviewTarget>, target: PrReviewTarget) {
    if !targets.iter().any(|existing| existing.id == target.id) {
        targets.push(target);
    }
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

fn id(value: impl Into<String>) -> RuntimeResult<Id> {
    Ok(Id::new(value)?)
}

fn validation_error(reason: impl Into<String>) -> RuntimeError {
    RuntimeError::workflow_construction(WORKFLOW_NAME, reason)
}
