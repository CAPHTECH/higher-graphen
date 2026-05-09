//! Structure mappings, composition, preservation checks, lost structure, and
//! distortion for HigherGraphen.

use higher_graphen_core::{Id, Provenance, Severity};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Source-to-target cell identifier mapping for a morphism.
pub type CellMapping = BTreeMap<Id, Id>;

/// Source-to-target relation identifier mapping for a morphism.
pub type RelationMapping = BTreeMap<Id, Id>;

/// Product-neutral category for the kind of transformation a morphism records.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MorphismType {
    /// A source structure is summarized into a coarser target structure.
    Abstraction,
    /// A source structure is made more specific in the target structure.
    Refinement,
    /// A source structure is translated into another representation.
    Translation,
    /// A source structure is projected into a selected target view.
    Projection,
    /// A source structure is lifted into a richer target structure.
    Lift,
    /// A source structure is migrated into a replacement target structure.
    Migration,
    /// A source structure is interpreted using another structural vocabulary.
    Interpretation,
    /// A downstream-owned transformation category.
    Custom(String),
}

/// Source structure that is not preserved by a morphism.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LostStructure {
    /// Source element that is lost.
    pub source_element_id: Id,
    /// Product-neutral explanation for the loss.
    pub reason: String,
    /// Impact classification for the loss.
    pub severity: Severity,
}

/// Difference introduced between a source element and its mapped target.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Distortion {
    /// Source element affected by the distortion.
    pub source_element_id: Id,
    /// Target element that carries the distorted representation.
    pub target_element_id: Id,
    /// Product-neutral explanation of the distortion.
    pub description: String,
    /// Impact classification for the distortion.
    pub severity: Severity,
}

/// A structure-preserving or structure-changing mapping between two spaces.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Morphism {
    /// Stable morphism identifier.
    pub id: Id,
    /// Source space identifier.
    pub source_space_id: Id,
    /// Target space identifier.
    pub target_space_id: Id,
    /// Human-readable morphism name.
    pub name: String,
    /// Product-neutral transformation category.
    pub morphism_type: MorphismType,
    /// Explicit source-cell to target-cell mappings.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub cell_mapping: CellMapping,
    /// Explicit source-relation to target-relation mappings.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub relation_mapping: RelationMapping,
    /// Invariants known to be preserved by this morphism.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preserved_invariant_ids: Vec<Id>,
    /// Source elements known to be lost by this morphism.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lost_structure: Vec<LostStructure>,
    /// Distortions known to be introduced by this morphism.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub distortion: Vec<Distortion>,
    /// Morphism identifiers declared compatible by metadata.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub composable_with: Vec<Id>,
    /// Source and review metadata for this morphism.
    pub provenance: Provenance,
}

/// Deterministic preservation check result for selected invariant IDs.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreservationReport {
    /// Selected invariant IDs found in the morphism preserved set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preserved: Vec<Id>,
    /// Selected invariant IDs absent from the morphism preserved set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub violated: Vec<Id>,
    /// Lost structure recorded on the checked morphism.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lost_structure: Vec<LostStructure>,
    /// Distortion recorded on the checked morphism.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub distortion: Vec<Distortion>,
}

/// Explicit mapping coverage for a two-morphism composition.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionCoverage {
    /// Intermediate cell IDs produced by the first morphism but not accepted by the second.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmapped_cell_intermediate_ids: Vec<Id>,
    /// Intermediate relation IDs produced by the first morphism but not accepted by the second.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmapped_relation_intermediate_ids: Vec<Id>,
}

impl CompositionCoverage {
    /// Returns true when every explicit first-morphism mapping can continue through the second.
    pub fn is_complete(&self) -> bool {
        self.unmapped_cell_intermediate_ids.is_empty()
            && self.unmapped_relation_intermediate_ids.is_empty()
    }
}

/// Stable obstruction type emitted by checked composition failures.
///
/// The value matches `ObstructionType::FailedComposition` without coupling this
/// crate to the obstruction package.
pub const FAILED_COMPOSITION_OBSTRUCTION_TYPE: &str = "failed_composition";

/// Kind of explicit mapping gap that prevents checked composition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailedCompositionFindingKind {
    /// A cell produced by the first morphism is not accepted by the second.
    UnmappedIntermediateCell,
    /// A relation produced by the first morphism is not accepted by the second.
    UnmappedIntermediateRelation,
}

/// First-class witness for a failed checked composition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FailedCompositionFinding {
    /// Stable obstruction type for downstream obstruction projection.
    pub obstruction_type: String,
    /// Specific mapping-gap category.
    pub finding_type: FailedCompositionFindingKind,
    /// Identifier of the first morphism in the attempted composition.
    pub first_morphism_id: Id,
    /// Identifier of the second morphism in the attempted composition.
    pub second_morphism_id: Id,
    /// Source cell or relation whose mapped intermediate cannot continue.
    pub source_element_id: Id,
    /// Intermediate cell or relation missing from the second morphism.
    pub intermediate_element_id: Id,
}

/// Stable obstruction emitted by explicit pullback-candidate extraction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PullbackObstructionType {
    /// The two morphisms do not map into the same target space.
    IncompatibleTargetSpace,
    /// At least one explicit mapping has no partner with the same target.
    PullbackIncomplete,
}

/// Structured pullback extraction obstruction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PullbackObstruction {
    /// Obstruction category.
    pub obstruction_type: PullbackObstructionType,
    /// Human-readable diagnostic.
    pub reason: String,
}

/// Pair of source cells that map to the same target cell.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PullbackCellMatch {
    /// Source cell from the left morphism.
    pub left_cell_id: Id,
    /// Source cell from the right morphism.
    pub right_cell_id: Id,
    /// Common target cell.
    pub target_cell_id: Id,
}

/// Pair of source relations that map to the same target relation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PullbackRelationMatch {
    /// Source relation from the left morphism.
    pub left_relation_id: Id,
    /// Source relation from the right morphism.
    pub right_relation_id: Id,
    /// Common target relation.
    pub target_relation_id: Id,
}

/// Deterministic explicit pullback candidate over two morphism mappings.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExplicitPullbackReport {
    /// Left morphism used by the construction.
    pub left_morphism_id: Id,
    /// Right morphism used by the construction.
    pub right_morphism_id: Id,
    /// Source space from the left morphism.
    pub left_source_space_id: Id,
    /// Source space from the right morphism.
    pub right_source_space_id: Id,
    /// Shared target space when the two morphisms are compatible.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_space_id: Option<Id>,
    /// Cell pairs that agree after mapping to the target.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cell_matches: Vec<PullbackCellMatch>,
    /// Relation pairs that agree after mapping to the target.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relation_matches: Vec<PullbackRelationMatch>,
    /// Left source cells with no right partner.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmatched_left_cell_ids: Vec<Id>,
    /// Right source cells with no left partner.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmatched_right_cell_ids: Vec<Id>,
    /// Left source relations with no right partner.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmatched_left_relation_ids: Vec<Id>,
    /// Right source relations with no left partner.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmatched_right_relation_ids: Vec<Id>,
    /// Explicit construction limitations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub information_loss: Vec<String>,
    /// Obstructions found while extracting the candidate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<PullbackObstruction>,
    /// Review status for this candidate report.
    #[serde(default)]
    pub review_status: higher_graphen_core::ReviewStatus,
}

impl ExplicitPullbackReport {
    /// Returns true when targets are compatible and all explicit mappings have partners.
    pub fn is_complete(&self) -> bool {
        self.obstructions.is_empty()
    }
}

/// Result of an explicit two-morphism composition attempt.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CompositionResult {
    /// The two morphisms were compatible and produced a composed morphism.
    Composed {
        /// The composed morphism from the first source space to the second target space.
        morphism: Box<Morphism>,
    },
    /// The first target space did not match the second source space.
    IncompatibleSpace {
        /// Identifier of the first morphism in the attempted composition.
        first_morphism_id: Id,
        /// Identifier of the second morphism in the attempted composition.
        second_morphism_id: Id,
        /// Target space identifier from the first morphism.
        first_target_space_id: Id,
        /// Source space identifier from the second morphism.
        second_source_space_id: Id,
    },
}

/// Result of a strict two-morphism composition attempt.
///
/// Unlike [`CompositionResult`], checked composition fails when compatible
/// spaces still have explicit first-morphism cell or relation mappings that
/// cannot continue through the second morphism.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CheckedCompositionResult {
    /// The two morphisms were compatible and all explicit mappings continued.
    Composed {
        /// The composed morphism from the first source space to the second target space.
        morphism: Box<Morphism>,
    },
    /// The first target space did not match the second source space.
    IncompatibleSpace {
        /// Identifier of the first morphism in the attempted composition.
        first_morphism_id: Id,
        /// Identifier of the second morphism in the attempted composition.
        second_morphism_id: Id,
        /// Target space identifier from the first morphism.
        first_target_space_id: Id,
        /// Source space identifier from the second morphism.
        second_source_space_id: Id,
    },
    /// Compatible spaces had explicit first-morphism mappings that could not continue.
    FailedComposition {
        /// Stable obstruction type matching `ObstructionType::FailedComposition`.
        obstruction_type: String,
        /// Coverage summary for the missing intermediate cells and relations.
        coverage: CompositionCoverage,
        /// Per-source witnesses for the missing intermediate cells and relations.
        findings: Vec<FailedCompositionFinding>,
    },
}

impl Morphism {
    /// Checks selected invariant IDs against this morphism's preserved set.
    ///
    /// The check is deterministic: selected IDs are deduplicated and returned
    /// in identifier order.
    pub fn check_preservation<I>(&self, invariant_ids: I) -> PreservationReport
    where
        I: IntoIterator<Item = Id>,
    {
        let known_preserved: BTreeSet<Id> = self.preserved_invariant_ids.iter().cloned().collect();
        let selected: BTreeSet<Id> = invariant_ids.into_iter().collect();
        let (preserved, violated) = partition_by_membership(selected, &known_preserved);

        PreservationReport {
            preserved,
            violated,
            lost_structure: self.lost_structure.clone(),
            distortion: self.distortion.clone(),
        }
    }

    /// Attempts to compose `self` followed by `second`.
    ///
    /// Composition succeeds only when `self.target_space_id` equals
    /// `second.source_space_id`. Metadata such as `composable_with` is carried
    /// by the model, but this deterministic MVP does not treat it as proof of
    /// compatibility.
    pub fn compose_with(
        &self,
        second: &Self,
        composed_id: Id,
        name: impl Into<String>,
        morphism_type: MorphismType,
        provenance: Provenance,
    ) -> CompositionResult {
        compose_morphisms(self, second, composed_id, name, morphism_type, provenance)
    }

    /// Strictly composes `self` followed by `second`.
    ///
    /// This preserves the space compatibility behavior of [`Self::compose_with`]
    /// and additionally reports unmapped intermediate cells or relations as
    /// failed-composition findings instead of returning a partial mapping.
    pub fn compose_checked_with(
        &self,
        second: &Self,
        composed_id: Id,
        name: impl Into<String>,
        morphism_type: MorphismType,
        provenance: Provenance,
    ) -> CheckedCompositionResult {
        compose_morphisms_checked(self, second, composed_id, name, morphism_type, provenance)
    }

    /// Reports explicit first-morphism mappings that cannot continue through `second`.
    pub fn composition_coverage_with(&self, second: &Self) -> CompositionCoverage {
        composition_coverage(self, second)
    }

    /// Reports strict-composition findings for explicit mappings that cannot continue.
    pub fn failed_composition_findings_with(&self, second: &Self) -> Vec<FailedCompositionFinding> {
        failed_composition_findings(self, second)
    }

    /// Extracts a finite explicit pullback candidate with another morphism.
    pub fn explicit_pullback_with(&self, right: &Self) -> ExplicitPullbackReport {
        explicit_pullback_candidate(self, right)
    }
}

/// Attempts to compose `first` followed by `second`.
///
/// Mapping composition follows only explicit mapping pairs:
/// `source -> intermediate` from `first` and `intermediate -> target` from
/// `second`. Unmatched intermediate IDs are not inferred.
pub fn compose_morphisms(
    first: &Morphism,
    second: &Morphism,
    composed_id: Id,
    name: impl Into<String>,
    morphism_type: MorphismType,
    provenance: Provenance,
) -> CompositionResult {
    if first.target_space_id != second.source_space_id {
        return CompositionResult::IncompatibleSpace {
            first_morphism_id: first.id.clone(),
            second_morphism_id: second.id.clone(),
            first_target_space_id: first.target_space_id.clone(),
            second_source_space_id: second.source_space_id.clone(),
        };
    }

    let cell_mapping = compose_mapping_parts(&first.cell_mapping, &second.cell_mapping).mapping;
    let relation_mapping =
        compose_mapping_parts(&first.relation_mapping, &second.relation_mapping).mapping;

    CompositionResult::Composed {
        morphism: Box::new(composed_morphism(
            first,
            second,
            ComposedMorphismSpec {
                composed_id,
                name: name.into(),
                morphism_type,
                provenance,
                cell_mapping,
                relation_mapping,
            },
        )),
    }
}

/// Strictly attempts to compose `first` followed by `second`.
///
/// Compatible spaces are not sufficient for checked composition: every explicit
/// `source -> intermediate` cell and relation mapping from `first` must have a
/// matching `intermediate -> target` mapping in `second`. Missing continuations
/// return [`CheckedCompositionResult::FailedComposition`] with structured
/// findings and no partial composed morphism.
pub fn compose_morphisms_checked(
    first: &Morphism,
    second: &Morphism,
    composed_id: Id,
    name: impl Into<String>,
    morphism_type: MorphismType,
    provenance: Provenance,
) -> CheckedCompositionResult {
    if first.target_space_id != second.source_space_id {
        return CheckedCompositionResult::IncompatibleSpace {
            first_morphism_id: first.id.clone(),
            second_morphism_id: second.id.clone(),
            first_target_space_id: first.target_space_id.clone(),
            second_source_space_id: second.source_space_id.clone(),
        };
    }

    let cell_composition = compose_mapping_parts(&first.cell_mapping, &second.cell_mapping);
    let relation_composition =
        compose_mapping_parts(&first.relation_mapping, &second.relation_mapping);
    let coverage = coverage_from_mapping_compositions(&cell_composition, &relation_composition);
    let findings =
        findings_from_mapping_compositions(first, second, &cell_composition, &relation_composition);

    if !findings.is_empty() {
        return CheckedCompositionResult::FailedComposition {
            obstruction_type: FAILED_COMPOSITION_OBSTRUCTION_TYPE.to_owned(),
            coverage,
            findings,
        };
    }

    CheckedCompositionResult::Composed {
        morphism: Box::new(composed_morphism(
            first,
            second,
            ComposedMorphismSpec {
                composed_id,
                name: name.into(),
                morphism_type,
                provenance,
                cell_mapping: cell_composition.mapping,
                relation_mapping: relation_composition.mapping,
            },
        )),
    }
}

/// Reports explicit first-morphism mappings that would be omitted by composition.
///
/// Space compatibility is intentionally not checked here. Use this diagnostic
/// before or after [`compose_morphisms`] to explain which intermediate IDs
/// prevented complete mapping composition.
pub fn composition_coverage(first: &Morphism, second: &Morphism) -> CompositionCoverage {
    let cell_composition = compose_mapping_parts(&first.cell_mapping, &second.cell_mapping);
    let relation_composition =
        compose_mapping_parts(&first.relation_mapping, &second.relation_mapping);

    coverage_from_mapping_compositions(&cell_composition, &relation_composition)
}

/// Reports strict-composition findings for explicit mappings that would fail composition.
///
/// Space compatibility is intentionally not checked here, matching
/// [`composition_coverage`]. Use [`compose_morphisms_checked`] when both space
/// compatibility and mapping completeness should be enforced together.
pub fn failed_composition_findings(
    first: &Morphism,
    second: &Morphism,
) -> Vec<FailedCompositionFinding> {
    let cell_composition = compose_mapping_parts(&first.cell_mapping, &second.cell_mapping);
    let relation_composition =
        compose_mapping_parts(&first.relation_mapping, &second.relation_mapping);

    findings_from_mapping_compositions(first, second, &cell_composition, &relation_composition)
}

/// Extracts common mapped substructure for two morphisms with a shared target.
///
/// The construction is finite and explicit: a left source and right source
/// match only when both are mapped to the same target identifier. Missing
/// mappings remain visible as unmatched identifiers; the report is a candidate,
/// not an accepted categorical universal property.
pub fn explicit_pullback_candidate(left: &Morphism, right: &Morphism) -> ExplicitPullbackReport {
    let compatible_target = left.target_space_id == right.target_space_id;
    let (cell_matches, unmatched_left_cell_ids, unmatched_right_cell_ids) =
        pullback_matches(&left.cell_mapping, &right.cell_mapping).into_parts();
    let (relation_matches, unmatched_left_relation_ids, unmatched_right_relation_ids) =
        pullback_matches(&left.relation_mapping, &right.relation_mapping).into_parts();
    let mut obstructions = Vec::new();

    if !compatible_target {
        obstructions.push(PullbackObstruction {
            obstruction_type: PullbackObstructionType::IncompatibleTargetSpace,
            reason: format!(
                "left target space {} differs from right target space {}",
                left.target_space_id, right.target_space_id
            ),
        });
    }
    if !unmatched_left_cell_ids.is_empty()
        || !unmatched_right_cell_ids.is_empty()
        || !unmatched_left_relation_ids.is_empty()
        || !unmatched_right_relation_ids.is_empty()
    {
        obstructions.push(PullbackObstruction {
            obstruction_type: PullbackObstructionType::PullbackIncomplete,
            reason: "some explicit mappings have no partner with the same target".to_owned(),
        });
    }

    ExplicitPullbackReport {
        left_morphism_id: left.id.clone(),
        right_morphism_id: right.id.clone(),
        left_source_space_id: left.source_space_id.clone(),
        right_source_space_id: right.source_space_id.clone(),
        target_space_id: compatible_target.then(|| left.target_space_id.clone()),
        cell_matches: cell_matches
            .into_iter()
            .map(|matched| PullbackCellMatch {
                left_cell_id: matched.left_source_id,
                right_cell_id: matched.right_source_id,
                target_cell_id: matched.target_id,
            })
            .collect(),
        relation_matches: relation_matches
            .into_iter()
            .map(|matched| PullbackRelationMatch {
                left_relation_id: matched.left_source_id,
                right_relation_id: matched.right_source_id,
                target_relation_id: matched.target_id,
            })
            .collect(),
        unmatched_left_cell_ids,
        unmatched_right_cell_ids,
        unmatched_left_relation_ids,
        unmatched_right_relation_ids,
        information_loss: vec![
            "only explicit mapping equality is considered".to_owned(),
            "universal property is not proven by this finite candidate report".to_owned(),
        ],
        obstructions,
        review_status: higher_graphen_core::ReviewStatus::Unreviewed,
    }
}

struct ComposedMorphismSpec {
    composed_id: Id,
    name: String,
    morphism_type: MorphismType,
    provenance: Provenance,
    cell_mapping: CellMapping,
    relation_mapping: RelationMapping,
}

fn composed_morphism(first: &Morphism, second: &Morphism, spec: ComposedMorphismSpec) -> Morphism {
    Morphism {
        id: spec.composed_id,
        source_space_id: first.source_space_id.clone(),
        target_space_id: second.target_space_id.clone(),
        name: spec.name,
        morphism_type: spec.morphism_type,
        cell_mapping: spec.cell_mapping,
        relation_mapping: spec.relation_mapping,
        preserved_invariant_ids: intersect_ids(
            &first.preserved_invariant_ids,
            &second.preserved_invariant_ids,
        ),
        lost_structure: concat_records(&first.lost_structure, &second.lost_structure),
        distortion: concat_records(&first.distortion, &second.distortion),
        composable_with: Vec::new(),
        provenance: spec.provenance,
    }
}

fn coverage_from_mapping_compositions(
    cell_composition: &MappingComposition,
    relation_composition: &MappingComposition,
) -> CompositionCoverage {
    CompositionCoverage {
        unmapped_cell_intermediate_ids: unmapped_intermediate_ids(cell_composition),
        unmapped_relation_intermediate_ids: unmapped_intermediate_ids(relation_composition),
    }
}

fn findings_from_mapping_compositions(
    first: &Morphism,
    second: &Morphism,
    cell_composition: &MappingComposition,
    relation_composition: &MappingComposition,
) -> Vec<FailedCompositionFinding> {
    cell_composition
        .unmapped
        .iter()
        .map(|gap| {
            failed_composition_finding(
                first,
                second,
                FailedCompositionFindingKind::UnmappedIntermediateCell,
                gap,
            )
        })
        .chain(relation_composition.unmapped.iter().map(|gap| {
            failed_composition_finding(
                first,
                second,
                FailedCompositionFindingKind::UnmappedIntermediateRelation,
                gap,
            )
        }))
        .collect()
}

fn failed_composition_finding(
    first: &Morphism,
    second: &Morphism,
    finding_type: FailedCompositionFindingKind,
    gap: &MappingGap,
) -> FailedCompositionFinding {
    FailedCompositionFinding {
        obstruction_type: FAILED_COMPOSITION_OBSTRUCTION_TYPE.to_owned(),
        finding_type,
        first_morphism_id: first.id.clone(),
        second_morphism_id: second.id.clone(),
        source_element_id: gap.source_element_id.clone(),
        intermediate_element_id: gap.intermediate_element_id.clone(),
    }
}

fn unmapped_intermediate_ids(composition: &MappingComposition) -> Vec<Id> {
    composition
        .unmapped
        .iter()
        .map(|gap| gap.intermediate_element_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn compose_mapping_parts(
    first: &BTreeMap<Id, Id>,
    second: &BTreeMap<Id, Id>,
) -> MappingComposition {
    let mut mapping = BTreeMap::new();
    let mut unmapped = Vec::new();

    for (source_id, intermediate_id) in first {
        if let Some(target_id) = second.get(intermediate_id) {
            mapping.insert(source_id.clone(), target_id.clone());
        } else {
            unmapped.push(MappingGap {
                source_element_id: source_id.clone(),
                intermediate_element_id: intermediate_id.clone(),
            });
        }
    }

    MappingComposition { mapping, unmapped }
}

fn pullback_matches(left: &BTreeMap<Id, Id>, right: &BTreeMap<Id, Id>) -> PullbackMatches {
    let mut right_by_target = BTreeMap::<Id, Vec<Id>>::new();
    for (right_source_id, target_id) in right {
        right_by_target
            .entry(target_id.clone())
            .or_default()
            .push(right_source_id.clone());
    }

    let mut left_by_target = BTreeMap::<Id, Vec<Id>>::new();
    for (left_source_id, target_id) in left {
        left_by_target
            .entry(target_id.clone())
            .or_default()
            .push(left_source_id.clone());
    }

    let mut matches = Vec::new();
    let mut unmatched_left_ids = BTreeSet::new();
    for (left_source_id, target_id) in left {
        if let Some(right_source_ids) = right_by_target.get(target_id) {
            for right_source_id in right_source_ids {
                matches.push(PullbackMappingMatch {
                    left_source_id: left_source_id.clone(),
                    right_source_id: right_source_id.clone(),
                    target_id: target_id.clone(),
                });
            }
        } else {
            unmatched_left_ids.insert(left_source_id.clone());
        }
    }

    let mut unmatched_right_ids = BTreeSet::new();
    for (right_source_id, target_id) in right {
        if !left_by_target.contains_key(target_id) {
            unmatched_right_ids.insert(right_source_id.clone());
        }
    }

    PullbackMatches {
        matches,
        unmatched_left_ids: unmatched_left_ids.into_iter().collect(),
        unmatched_right_ids: unmatched_right_ids.into_iter().collect(),
    }
}

#[derive(Debug)]
struct MappingComposition {
    mapping: BTreeMap<Id, Id>,
    unmapped: Vec<MappingGap>,
}

#[derive(Debug)]
struct MappingGap {
    source_element_id: Id,
    intermediate_element_id: Id,
}

#[derive(Debug)]
struct PullbackMatches {
    matches: Vec<PullbackMappingMatch>,
    unmatched_left_ids: Vec<Id>,
    unmatched_right_ids: Vec<Id>,
}

impl PullbackMatches {
    fn into_parts(self) -> (Vec<PullbackMappingMatch>, Vec<Id>, Vec<Id>) {
        (
            self.matches,
            self.unmatched_left_ids,
            self.unmatched_right_ids,
        )
    }
}

#[derive(Debug)]
struct PullbackMappingMatch {
    left_source_id: Id,
    right_source_id: Id,
    target_id: Id,
}

fn intersect_ids(first: &[Id], second: &[Id]) -> Vec<Id> {
    let first_ids: BTreeSet<Id> = first.iter().cloned().collect();
    let second_ids: BTreeSet<Id> = second.iter().cloned().collect();

    first_ids.intersection(&second_ids).cloned().collect()
}

fn concat_records<T: Clone>(first: &[T], second: &[T]) -> Vec<T> {
    first.iter().chain(second.iter()).cloned().collect()
}

fn partition_by_membership(
    selected: BTreeSet<Id>,
    known_preserved: &BTreeSet<Id>,
) -> (Vec<Id>, Vec<Id>) {
    selected
        .into_iter()
        .partition(|invariant_id| known_preserved.contains(invariant_id))
}

#[cfg(test)]
mod tests;
