use super::{id, push_unique, slug};
use crate::error::RuntimeResult;
use crate::pr_review_reports::{
    PrReviewTarget, PrReviewTargetInputChangedFile, PrReviewTargetInputDependencyEdge,
    PrReviewTargetInputDocument, PrReviewTargetInputRiskSignal, PrReviewTargetInputSymbol,
    PrReviewTargetInputTest, PrReviewTargetLocation, PrReviewTargetObstruction,
    PrReviewTargetObstructionType, PrReviewTargetType,
};
use higher_graphen_core::{Id, ReviewStatus, Severity};
use higher_graphen_structure::space::{
    CoverageCandidate, DominanceAnalysis, WeightedCoverageSelector, WeightedUniverseElement,
};

pub(super) fn recommend_review_targets(
    input: &PrReviewTargetInputDocument,
) -> RuntimeResult<Vec<PrReviewTarget>> {
    let mut targets = Vec::new();
    for signal in &input.signals {
        if !should_expand_signal_to_sources(signal) {
            push_unique_target(&mut targets, cross_cutting_target(input, signal)?);
            continue;
        }

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
    order_review_targets_by_signal_coverage(input, &mut targets);
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
    Ok(PrReviewTarget {
        id: target_id(signal, &symbol.id)?,
        target_type: PrReviewTargetType::Symbol,
        target_ref: symbol.id.to_string(),
        title: format!("Review {}", symbol.name),
        rationale: signal.summary.clone(),
        evidence_ids: target_evidence_ids(signal, Some(&symbol.id)),
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
        rationale: signal.summary.clone(),
        evidence_ids: target_evidence_ids(signal, Some(&file.id)),
        location: Some(PrReviewTargetLocation {
            path: file.path.clone(),
            line_start: None,
            line_end: None,
            symbol_id: None,
        }),
        suggested_questions: questions_for_signal(signal),
        related_target_ids: Vec::new(),
        severity: recommended_file_severity(signal, &file.path),
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
        rationale: signal.summary.clone(),
        evidence_ids: target_evidence_ids(signal, Some(&edge.id)),
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
        rationale: signal.summary.clone(),
        evidence_ids: target_evidence_ids(signal, Some(&test.id)),
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
        evidence_ids: target_evidence_ids(signal, None),
        location: None,
        suggested_questions: questions_for_signal(signal),
        related_target_ids: Vec::new(),
        severity: recommended_severity(signal),
        confidence: signal.confidence,
        review_status: ReviewStatus::Unreviewed,
    })
}

pub(super) fn review_obstructions(
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

fn recommended_file_severity(signal: &PrReviewTargetInputRiskSignal, path: &str) -> Severity {
    let severity = recommended_severity(signal);
    if matches!(
        signal.signal_type,
        crate::pr_review_reports::PrReviewTargetRiskSignalType::DependencyChange
    ) && is_supporting_contract_file(path)
        && severity > Severity::Medium
    {
        Severity::Medium
    } else {
        severity
    }
}

fn is_supporting_contract_file(path: &str) -> bool {
    path.starts_with("docs/")
        || path.starts_with("skills/")
        || path.ends_with("README.md")
        || path.ends_with(".example.json")
}

fn should_expand_signal_to_sources(signal: &PrReviewTargetInputRiskSignal) -> bool {
    use crate::pr_review_reports::PrReviewTargetRiskSignalType;

    !matches!(
        signal.signal_type,
        PrReviewTargetRiskSignalType::LargeChange | PrReviewTargetRiskSignalType::GeneratedCode
    )
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

fn target_evidence_ids(signal: &PrReviewTargetInputRiskSignal, target_id: Option<&Id>) -> Vec<Id> {
    let mut ids = Vec::new();
    if let Some(target_id) = target_id {
        push_unique(&mut ids, target_id.clone());
    } else {
        for source_id in &signal.source_ids {
            push_unique(&mut ids, source_id.clone());
        }
    }
    push_unique(&mut ids, signal.id.clone());
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

fn file_label(path: &str) -> String {
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_owned()
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

fn push_unique_target(targets: &mut Vec<PrReviewTarget>, target: PrReviewTarget) {
    if let Some(existing) = targets
        .iter_mut()
        .find(|existing| same_review_target(existing, &target))
    {
        merge_review_target(existing, target);
    } else {
        targets.push(target);
    }
}

fn same_review_target(left: &PrReviewTarget, right: &PrReviewTarget) -> bool {
    left.target_type == right.target_type && left.target_ref == right.target_ref
}

fn merge_review_target(existing: &mut PrReviewTarget, incoming: PrReviewTarget) {
    if existing.severity < incoming.severity {
        existing.severity = incoming.severity;
    }
    if existing.confidence < incoming.confidence {
        existing.confidence = incoming.confidence;
    }
    if existing.location.is_none() {
        existing.location = incoming.location;
    }
    existing.rationale = merge_rationale(&existing.rationale, &incoming.rationale);
    for evidence_id in incoming.evidence_ids {
        push_unique(&mut existing.evidence_ids, evidence_id);
    }
    for question in incoming.suggested_questions {
        if !existing.suggested_questions.contains(&question) {
            existing.suggested_questions.push(question);
        }
    }
    for related_target_id in incoming.related_target_ids {
        push_unique(&mut existing.related_target_ids, related_target_id);
    }
}

fn merge_rationale(existing: &str, incoming: &str) -> String {
    let mut parts = rationale_parts(existing);
    for part in rationale_parts(incoming) {
        if !parts.contains(&part) {
            parts.push(part);
        }
    }

    if parts.len() == 1 {
        parts.remove(0)
    } else {
        format!("Multiple signals apply: {}", parts.join("; "))
    }
}

fn rationale_parts(value: &str) -> Vec<String> {
    const PREFIX: &str = "Multiple signals apply: ";
    value
        .strip_prefix(PREFIX)
        .unwrap_or(value)
        .split("; ")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

fn order_review_targets_by_signal_coverage(
    input: &PrReviewTargetInputDocument,
    targets: &mut [PrReviewTarget],
) {
    let weighted_signals = input
        .signals
        .iter()
        .map(|signal| WeightedUniverseElement::new(signal.id.clone(), review_signal_weight(signal)))
        .collect::<Vec<_>>();
    let signal_ids = weighted_signals
        .iter()
        .map(|signal| signal.id.clone())
        .collect::<Vec<_>>();
    if signal_ids.is_empty() || targets.is_empty() {
        return;
    }

    let candidates = targets
        .iter()
        .map(|target| {
            CoverageCandidate::new(target.id.clone(), target_signal_ids(target, &signal_ids))
                .with_priority(review_target_priority(target))
        })
        .collect::<Vec<_>>();
    let dominated_ids = DominanceAnalysis::new(candidates.clone())
        .analyze()
        .dominated_ids
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let selected_ids = WeightedCoverageSelector::new(weighted_signals)
        .with_candidates(candidates)
        .select()
        .selected_ids;

    targets.sort_by(|left, right| {
        selected_position(&selected_ids, &left.id)
            .cmp(&selected_position(&selected_ids, &right.id))
            .then_with(|| {
                dominated_ids
                    .contains(&left.id)
                    .cmp(&dominated_ids.contains(&right.id))
            })
            .then_with(|| right.severity.cmp(&left.severity))
            .then_with(|| {
                right
                    .confidence
                    .partial_cmp(&left.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.target_ref.cmp(&right.target_ref))
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn review_signal_weight(signal: &PrReviewTargetInputRiskSignal) -> u32 {
    let severity = match signal.severity {
        Severity::Low => 1,
        Severity::Medium => 2,
        Severity::High => 4,
        Severity::Critical => 8,
    };
    let confidence = (signal.confidence.value() * 100.0).round() as u32;
    severity * confidence.max(1)
}

fn target_signal_ids(target: &PrReviewTarget, signal_ids: &[Id]) -> Vec<Id> {
    target
        .evidence_ids
        .iter()
        .filter(|evidence_id| signal_ids.contains(evidence_id))
        .cloned()
        .collect()
}

fn selected_position(selected_ids: &[Id], target_id: &Id) -> usize {
    selected_ids
        .iter()
        .position(|selected_id| selected_id == target_id)
        .unwrap_or(usize::MAX)
}

fn review_target_priority(target: &PrReviewTarget) -> u32 {
    let severity = match target.severity {
        Severity::Low => 1,
        Severity::Medium => 2,
        Severity::High => 3,
        Severity::Critical => 4,
    };
    let target_type = match target.target_type {
        PrReviewTargetType::Dependency => 4,
        PrReviewTargetType::Symbol => 3,
        PrReviewTargetType::File => 2,
        PrReviewTargetType::Test => 2,
        PrReviewTargetType::Documentation => 1,
        PrReviewTargetType::CrossCutting => 1,
    };
    severity * 10 + target_type
}
