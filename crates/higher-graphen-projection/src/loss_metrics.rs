//! Deterministic information-loss metrics for projection results.

use crate::{InformationLoss, ProjectionOutput, ProjectionResult};
use higher_graphen_core::{Id, Severity};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Projection loss metric families supported by this kernel.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionLossMetricKind {
    /// Finite structural metrics computed from output items and source ids.
    FiniteStructural,
}

/// Basis used to compute `ProjectionLossMetric::source_cardinality`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionSourceCardinalityBasis {
    /// Cardinality came from the explicit eligible source universe.
    EligibleSourceUniverse,
    /// Cardinality came from the result-level represented source union.
    RepresentedSourceUnion,
}

/// Finite structural loss measurements for a projection result.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionLossMetric {
    /// Projection identifier for the measured result.
    pub projection_id: Id,
    /// Metric family used for this record.
    pub metric_kind: ProjectionLossMetricKind,
    /// Basis used for `source_cardinality`.
    pub source_cardinality_basis: ProjectionSourceCardinalityBasis,
    /// Number of eligible sources when supplied, otherwise represented sources.
    pub source_cardinality: usize,
    /// Number of deterministic output items in the result.
    pub projected_cardinality: usize,
    /// Number of unordered traced source pairs collapsed into one output item.
    pub collapsed_pair_count: usize,
    /// Number of traced source pairs that never co-occur in one output item.
    pub distinguished_pair_count: usize,
    /// Eligible source identifiers absent from the result-level represented set.
    pub omitted_source_ids: Vec<Id>,
    /// Fraction of traced sources that appear in two or more output items.
    pub ambiguity_score: f64,
    /// Sorted union of source identifiers covered by declared information loss.
    pub declared_loss_source_ids: Vec<Id>,
}

/// Source group collapsed into a single traced projection output item.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionCollapsedSourceGroup {
    /// Deterministic output-item identifier.
    pub item_id: String,
    /// Sorted source identifiers collapsed into the item.
    pub source_ids: Vec<Id>,
}

/// Review-signal obstruction emitted by projection loss measurement.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionLossObstruction {
    /// Measurable collapse or omission is not fully covered by declared loss.
    UndeclaredProjectionLoss,
    /// A source appears in multiple output items without declared loss coverage.
    AmbiguousProjectionOutput,
    /// An output item lacks per-item source attribution.
    SourceTraceMissing,
    /// Per-item loss metrics cannot be verified for the output kind.
    UnsupportedLossMetric,
}

/// Ambiguity and review-signal details for a projection result.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionAmbiguityReport {
    /// Projection identifier for the measured result.
    pub projection_id: Id,
    /// Sorted output-item identifiers that share a source with another item.
    pub ambiguous_output_ids: Vec<String>,
    /// Traced output items that collapse more than one source.
    pub collapsed_source_groups: Vec<ProjectionCollapsedSourceGroup>,
    /// Sorted source identifiers involved in measurable undeclared loss.
    pub missing_loss_declarations: Vec<Id>,
    /// Review severity inferred from the emitted obstructions.
    pub risk_severity: Severity,
    /// Structured review signals discovered by the kernel.
    pub obstructions: Vec<ProjectionLossObstruction>,
}

/// Complete deterministic projection loss measurement report.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionLossReport {
    /// Finite structural metric values.
    pub metric: ProjectionLossMetric,
    /// Ambiguity details and review-signal obstructions.
    pub ambiguity: ProjectionAmbiguityReport,
}

struct OutputItem {
    id: String,
    source_ids: Option<Vec<Id>>,
}

/// Measures finite structural information loss for a projection result.
///
/// The function is pure and deterministic over its explicit inputs. It never
/// mutates the projection result and never changes review status.
#[must_use]
pub fn measure_projection_loss(
    result: &ProjectionResult,
    eligible_source_ids: &[Id],
) -> ProjectionLossReport {
    let represented = sorted_unique_ids(result.source_ids().iter().cloned());
    let eligible = sorted_unique_ids(eligible_source_ids.iter().cloned());
    let declared_loss_source_ids = declared_loss_source_ids(result.information_loss());
    let items = output_items(result.output());
    let has_untraced_attributable_item = has_untraced_attributable_item(result.output(), &items);
    let unsupported_per_item_metrics = has_untraced_output_kind(result.output());

    let traced_sources = traced_sources(&items);
    let source_cardinality_basis = if eligible.is_empty() {
        ProjectionSourceCardinalityBasis::RepresentedSourceUnion
    } else {
        ProjectionSourceCardinalityBasis::EligibleSourceUniverse
    };
    let source_cardinality = if eligible.is_empty() {
        represented.len()
    } else {
        eligible.len()
    };
    let omitted_source_ids = if eligible.is_empty() {
        Vec::new()
    } else {
        difference(&eligible, &represented)
    };

    let collapsed_source_groups = collapsed_source_groups(&items);
    let collapsed_pairs = collapsed_pairs(&collapsed_source_groups);
    let collapsed_pair_count = collapsed_pairs.len();
    let traced_pair_count = pair_count(traced_sources.len());
    let distinguished_pair_count = traced_pair_count.saturating_sub(collapsed_pair_count);

    let source_to_items = source_to_items(&items);
    let ambiguous_source_ids = ambiguous_source_ids(&source_to_items);
    let ambiguous_output_ids = ambiguous_output_ids(&source_to_items, &ambiguous_source_ids);
    let ambiguity_score = if traced_sources.is_empty() {
        0.0
    } else {
        ambiguous_source_ids.len() as f64 / traced_sources.len() as f64
    };

    let declared = declared_loss_source_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let missing_from_collapse = missing_collapsed_loss_sources(&collapsed_source_groups, &declared);
    let missing_from_omission = missing_sources(&omitted_source_ids, &declared);
    let missing_from_ambiguity = missing_sources(&ambiguous_source_ids, &declared);

    let mut undeclared_missing = missing_from_collapse;
    undeclared_missing.extend(missing_from_omission);

    let mut missing_loss_declarations = undeclared_missing.clone();
    missing_loss_declarations.extend(missing_from_ambiguity.iter().cloned());

    let obstructions = obstructions(
        !undeclared_missing.is_empty(),
        !missing_from_ambiguity.is_empty(),
        has_untraced_attributable_item,
        unsupported_per_item_metrics,
    );
    let risk_severity = risk_severity(&obstructions);

    ProjectionLossReport {
        metric: ProjectionLossMetric {
            projection_id: result.projection_id().clone(),
            metric_kind: ProjectionLossMetricKind::FiniteStructural,
            source_cardinality_basis,
            source_cardinality,
            projected_cardinality: items.len(),
            collapsed_pair_count,
            distinguished_pair_count,
            omitted_source_ids,
            ambiguity_score,
            declared_loss_source_ids,
        },
        ambiguity: ProjectionAmbiguityReport {
            projection_id: result.projection_id().clone(),
            ambiguous_output_ids,
            collapsed_source_groups,
            missing_loss_declarations: missing_loss_declarations.into_iter().collect(),
            risk_severity,
            obstructions,
        },
    }
}

fn output_items(output: &ProjectionOutput) -> Vec<OutputItem> {
    match output {
        ProjectionOutput::Sections { sections } => sections
            .iter()
            .enumerate()
            .map(|(index, section)| OutputItem {
                id: format!("section:{index}:{}", section.title),
                source_ids: Some(sorted_unique_ids(section.source_ids.iter().cloned())),
            })
            .collect(),
        ProjectionOutput::KeyValue { entries } => entries
            .iter()
            .enumerate()
            .map(|(index, entry)| OutputItem {
                id: format!("entry:{index}:{}", entry.key),
                source_ids: Some(sorted_unique_ids(entry.source_ids.iter().cloned())),
            })
            .collect(),
        ProjectionOutput::Text { .. } => vec![OutputItem {
            id: "text".to_owned(),
            source_ids: None,
        }],
        ProjectionOutput::Table { rows, .. } => rows
            .iter()
            .enumerate()
            .map(|(index, _row)| OutputItem {
                id: format!("row:{index}"),
                source_ids: None,
            })
            .collect(),
    }
}

fn has_untraced_output_kind(output: &ProjectionOutput) -> bool {
    matches!(
        output,
        ProjectionOutput::Text { .. } | ProjectionOutput::Table { .. }
    )
}

fn has_untraced_attributable_item(output: &ProjectionOutput, items: &[OutputItem]) -> bool {
    let carries_attribution = matches!(
        output,
        ProjectionOutput::Sections { .. } | ProjectionOutput::KeyValue { .. }
    );
    carries_attribution
        && (items.is_empty()
            || items
                .iter()
                .any(|item| matches!(&item.source_ids, Some(source_ids) if source_ids.is_empty())))
}

fn sorted_unique_ids<I>(ids: I) -> Vec<Id>
where
    I: IntoIterator<Item = Id>,
{
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn declared_loss_source_ids(information_loss: &[InformationLoss]) -> Vec<Id> {
    sorted_unique_ids(
        information_loss
            .iter()
            .flat_map(|loss| loss.source_ids().iter().cloned()),
    )
}

fn difference(left: &[Id], right: &[Id]) -> Vec<Id> {
    let right = right.iter().collect::<BTreeSet<_>>();
    left.iter()
        .filter(|id| !right.contains(id))
        .cloned()
        .collect()
}

fn traced_sources(items: &[OutputItem]) -> Vec<Id> {
    sorted_unique_ids(
        items
            .iter()
            .filter_map(|item| item.source_ids.as_ref())
            .flat_map(|source_ids| source_ids.iter().cloned()),
    )
}

fn collapsed_source_groups(items: &[OutputItem]) -> Vec<ProjectionCollapsedSourceGroup> {
    let mut groups = items
        .iter()
        .filter_map(|item| {
            let source_ids = item.source_ids.as_ref()?;
            if source_ids.len() > 1 {
                Some(ProjectionCollapsedSourceGroup {
                    item_id: item.id.clone(),
                    source_ids: source_ids.clone(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    groups.sort();
    groups
}

fn collapsed_pairs(groups: &[ProjectionCollapsedSourceGroup]) -> BTreeSet<(Id, Id)> {
    let mut pairs = BTreeSet::new();
    for group in groups {
        for left_index in 0..group.source_ids.len() {
            for right_index in (left_index + 1)..group.source_ids.len() {
                pairs.insert((
                    group.source_ids[left_index].clone(),
                    group.source_ids[right_index].clone(),
                ));
            }
        }
    }
    pairs
}

fn pair_count(count: usize) -> usize {
    count.saturating_mul(count.saturating_sub(1)) / 2
}

fn source_to_items(items: &[OutputItem]) -> BTreeMap<Id, BTreeSet<String>> {
    let mut source_to_items = BTreeMap::<Id, BTreeSet<String>>::new();
    for item in items {
        if let Some(source_ids) = item.source_ids.as_ref() {
            for source_id in source_ids {
                source_to_items
                    .entry(source_id.clone())
                    .or_default()
                    .insert(item.id.clone());
            }
        }
    }
    source_to_items
}

fn ambiguous_source_ids(source_to_items: &BTreeMap<Id, BTreeSet<String>>) -> Vec<Id> {
    source_to_items
        .iter()
        .filter(|(_source_id, item_ids)| item_ids.len() >= 2)
        .map(|(source_id, _item_ids)| source_id.clone())
        .collect()
}

fn ambiguous_output_ids(
    source_to_items: &BTreeMap<Id, BTreeSet<String>>,
    ambiguous_source_ids: &[Id],
) -> Vec<String> {
    let mut item_ids = BTreeSet::new();
    for source_id in ambiguous_source_ids {
        if let Some(ids) = source_to_items.get(source_id) {
            item_ids.extend(ids.iter().cloned());
        }
    }
    item_ids.into_iter().collect()
}

fn missing_collapsed_loss_sources(
    groups: &[ProjectionCollapsedSourceGroup],
    declared: &BTreeSet<Id>,
) -> BTreeSet<Id> {
    let mut missing = BTreeSet::new();
    for group in groups {
        if !group
            .source_ids
            .iter()
            .all(|source_id| declared.contains(source_id))
        {
            missing.extend(
                group
                    .source_ids
                    .iter()
                    .filter(|source_id| !declared.contains(*source_id))
                    .cloned(),
            );
        }
    }
    missing
}

fn missing_sources(source_ids: &[Id], declared: &BTreeSet<Id>) -> BTreeSet<Id> {
    source_ids
        .iter()
        .filter(|source_id| !declared.contains(*source_id))
        .cloned()
        .collect()
}

fn obstructions(
    has_undeclared_loss: bool,
    has_undeclared_ambiguity: bool,
    has_untraced_attributable_item: bool,
    unsupported_per_item_metrics: bool,
) -> Vec<ProjectionLossObstruction> {
    let mut obstructions = BTreeSet::new();
    if has_undeclared_loss {
        obstructions.insert(ProjectionLossObstruction::UndeclaredProjectionLoss);
    }
    if has_undeclared_ambiguity {
        obstructions.insert(ProjectionLossObstruction::AmbiguousProjectionOutput);
    }
    if has_untraced_attributable_item {
        obstructions.insert(ProjectionLossObstruction::SourceTraceMissing);
    }
    if unsupported_per_item_metrics {
        obstructions.insert(ProjectionLossObstruction::SourceTraceMissing);
        obstructions.insert(ProjectionLossObstruction::UnsupportedLossMetric);
    }
    obstructions.into_iter().collect()
}

fn risk_severity(obstructions: &[ProjectionLossObstruction]) -> Severity {
    if obstructions.iter().any(|obstruction| {
        matches!(
            obstruction,
            ProjectionLossObstruction::UndeclaredProjectionLoss
                | ProjectionLossObstruction::AmbiguousProjectionOutput
        )
    }) {
        Severity::High
    } else if obstructions.iter().any(|obstruction| {
        matches!(
            obstruction,
            ProjectionLossObstruction::SourceTraceMissing
                | ProjectionLossObstruction::UnsupportedLossMetric
        )
    }) {
        Severity::Medium
    } else {
        Severity::Low
    }
}
