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

/// Stable obstruction emitted by explicit pushout-candidate extraction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PushoutObstructionType {
    /// The two morphisms do not start from the same source space.
    IncompatibleSourceSpace,
    /// At least one source mapping has no partner on the other leg.
    PushoutIncomplete,
}

/// Structured pushout extraction obstruction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PushoutObstruction {
    /// Obstruction category.
    pub obstruction_type: PushoutObstructionType,
    /// Human-readable diagnostic.
    pub reason: String,
}

/// Candidate identification induced by two morphisms from the same source.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IdentifiedSourceGroup {
    /// Source element that induces the identification.
    pub source_element_id: Id,
    /// Target element from the left morphism.
    pub left_target_id: Id,
    /// Target element from the right morphism.
    pub right_target_id: Id,
}

/// Deterministic explicit pushout candidate over two morphism mappings.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExplicitPushoutReport {
    /// Candidate merged space identifier supplied by the caller.
    pub candidate_space_id: Id,
    /// Left morphism used by the construction.
    pub left_morphism_id: Id,
    /// Right morphism used by the construction.
    pub right_morphism_id: Id,
    /// Shared source space when the two morphisms are compatible.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_space_id: Option<Id>,
    /// Target space from the left morphism.
    pub left_target_space_id: Id,
    /// Target space from the right morphism.
    pub right_target_space_id: Id,
    /// Cell identifications induced by shared source cells.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identified_cell_groups: Vec<IdentifiedSourceGroup>,
    /// Relation identifications induced by shared source relations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identified_relation_groups: Vec<IdentifiedSourceGroup>,
    /// Left source cells with no right mapping.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmatched_left_cell_source_ids: Vec<Id>,
    /// Right source cells with no left mapping.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmatched_right_cell_source_ids: Vec<Id>,
    /// Left source relations with no right mapping.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmatched_left_relation_source_ids: Vec<Id>,
    /// Right source relations with no left mapping.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmatched_right_relation_source_ids: Vec<Id>,
    /// Explicit quotient losses created by this candidate construction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quotient_losses: Vec<String>,
    /// Obstructions found while extracting the candidate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<PushoutObstruction>,
    /// Review status for this candidate report.
    #[serde(default)]
    pub review_status: higher_graphen_core::ReviewStatus,
}

impl ExplicitPushoutReport {
    /// Returns true when sources are compatible and all explicit mappings have partners.
    pub fn is_complete(&self) -> bool {
        self.obstructions.is_empty()
    }

    /// Creates an empty candidate space shell for this pushout report.
    ///
    /// The shell carries the candidate identifier and name only. Cells,
    /// incidences, quotient losses, and inclusion morphisms remain reviewable
    /// report data and are not silently materialized as accepted structure.
    #[must_use]
    pub fn candidate_space_shell(&self, name: impl Into<String>) -> crate::space::Space {
        crate::space::Space::new(self.candidate_space_id.clone(), name)
    }
}

/// Stable obstruction emitted by finite diagram commutativity checks.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagramObstructionType {
    /// A path contains adjacent morphisms with incompatible spaces.
    IncompatiblePath,
    /// A path omits explicit mappings needed to compose fully.
    IncompletePath,
    /// The two paths do not have the same source and target spaces.
    IncompatibleBoundary,
    /// The two path mappings disagree.
    NonCommutativeDiagram,
}

/// Structured diagram-check obstruction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagramObstruction {
    /// Obstruction category.
    pub obstruction_type: DiagramObstructionType,
    /// Human-readable diagnostic.
    pub reason: String,
}

/// Explicit element category compared by diagram commutativity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagramElementKind {
    /// Cell mapping mismatch.
    Cell,
    /// Relation mapping mismatch.
    Relation,
}

/// Witness that two diagram paths disagree on an explicit source element.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NonCommutativeWitness {
    /// Element category.
    pub element_kind: DiagramElementKind,
    /// Source element being compared.
    pub source_element_id: Id,
    /// Target reached by the left path, when mapped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_target_id: Option<Id>,
    /// Target reached by the right path, when mapped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_target_id: Option<Id>,
}

/// Summary of one explicit path through a finite diagram.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagramPathSummary {
    /// Morphisms in path order.
    pub morphism_ids: Vec<Id>,
    /// Source space of the first morphism, when path is non-empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_space_id: Option<Id>,
    /// Target space of the last morphism, when path is non-empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_space_id: Option<Id>,
    /// Explicit cell mapping produced by path composition.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub cell_mapping: CellMapping,
    /// Explicit relation mapping produced by path composition.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub relation_mapping: RelationMapping,
    /// Explicit mapping coverage for this path.
    pub coverage: CompositionCoverage,
}

/// Deterministic commutativity check for two explicit morphism paths.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagramCommutativityReport {
    /// Left path summary.
    pub left_path: DiagramPathSummary,
    /// Right path summary.
    pub right_path: DiagramPathSummary,
    /// True only when both paths are complete, boundary-compatible, and mapping-equivalent.
    pub commutes: bool,
    /// Explicit source elements where path targets differ.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_commutative_witnesses: Vec<NonCommutativeWitness>,
    /// Obstructions found during checking.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<DiagramObstruction>,
    /// Explicit checking limitations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub information_loss: Vec<String>,
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

    /// Extracts a finite explicit pushout candidate with another morphism.
    pub fn explicit_pushout_with(
        &self,
        right: &Self,
        candidate_space_id: Id,
    ) -> ExplicitPushoutReport {
        explicit_pushout_candidate(self, right, candidate_space_id)
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

/// Extracts an explicit pushout-style merge candidate for two morphisms sharing a source.
///
/// The candidate identifies left and right targets that come from the same
/// source element. It does not construct a new space and does not accept the
/// quotient; losses and incompleteness remain explicit.
pub fn explicit_pushout_candidate(
    left: &Morphism,
    right: &Morphism,
    candidate_space_id: Id,
) -> ExplicitPushoutReport {
    let compatible_source = left.source_space_id == right.source_space_id;
    let (identified_cell_groups, unmatched_left_cell_source_ids, unmatched_right_cell_source_ids) =
        pushout_groups(&left.cell_mapping, &right.cell_mapping).into_parts();
    let (
        identified_relation_groups,
        unmatched_left_relation_source_ids,
        unmatched_right_relation_source_ids,
    ) = pushout_groups(&left.relation_mapping, &right.relation_mapping).into_parts();
    let mut obstructions = Vec::new();

    if !compatible_source {
        obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::IncompatibleSourceSpace,
            reason: format!(
                "left source space {} differs from right source space {}",
                left.source_space_id, right.source_space_id
            ),
        });
    }
    if !unmatched_left_cell_source_ids.is_empty()
        || !unmatched_right_cell_source_ids.is_empty()
        || !unmatched_left_relation_source_ids.is_empty()
        || !unmatched_right_relation_source_ids.is_empty()
    {
        obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::PushoutIncomplete,
            reason: "some explicit source mappings have no partner on the other leg".to_owned(),
        });
    }

    ExplicitPushoutReport {
        candidate_space_id,
        left_morphism_id: left.id.clone(),
        right_morphism_id: right.id.clone(),
        source_space_id: compatible_source.then(|| left.source_space_id.clone()),
        left_target_space_id: left.target_space_id.clone(),
        right_target_space_id: right.target_space_id.clone(),
        identified_cell_groups,
        identified_relation_groups,
        unmatched_left_cell_source_ids,
        unmatched_right_cell_source_ids,
        unmatched_left_relation_source_ids,
        unmatched_right_relation_source_ids,
        quotient_losses: vec![
            "identified target elements are quotient candidates, not accepted equivalences"
                .to_owned(),
            "invariant preservation across the quotient is not proven by this report".to_owned(),
        ],
        obstructions,
        review_status: higher_graphen_core::ReviewStatus::Unreviewed,
    }
}

/// Checks whether two explicit morphism paths commute.
///
/// This finite MVP compares only explicit cell and relation mappings produced
/// by path composition. Missing mappings and incompatible path boundaries are
/// retained as structured obstructions.
pub fn check_diagram_commutativity(
    left_path: &[Morphism],
    right_path: &[Morphism],
) -> DiagramCommutativityReport {
    let left = compose_path_summary(left_path);
    let right = compose_path_summary(right_path);
    let mut obstructions = Vec::new();
    obstructions.extend(path_boundary_obstructions("left", left_path));
    obstructions.extend(path_boundary_obstructions("right", right_path));
    obstructions.extend(path_obstructions("left", &left));
    obstructions.extend(path_obstructions("right", &right));

    if left.source_space_id != right.source_space_id
        || left.target_space_id != right.target_space_id
    {
        obstructions.push(DiagramObstruction {
            obstruction_type: DiagramObstructionType::IncompatibleBoundary,
            reason: "left and right paths do not share the same source and target spaces"
                .to_owned(),
        });
    }

    let non_commutative_witnesses = mapping_witnesses(
        DiagramElementKind::Cell,
        &left.cell_mapping,
        &right.cell_mapping,
    )
    .into_iter()
    .chain(mapping_witnesses(
        DiagramElementKind::Relation,
        &left.relation_mapping,
        &right.relation_mapping,
    ))
    .collect::<Vec<_>>();

    if !non_commutative_witnesses.is_empty() {
        obstructions.push(DiagramObstruction {
            obstruction_type: DiagramObstructionType::NonCommutativeDiagram,
            reason: "left and right path mappings disagree on explicit source elements".to_owned(),
        });
    }

    let commutes = obstructions.is_empty();

    DiagramCommutativityReport {
        left_path: left,
        right_path: right,
        commutes,
        non_commutative_witnesses,
        obstructions,
        information_loss: vec![
            "only explicit morphism mappings are compared".to_owned(),
            "unmapped source elements are reported as incomplete path coverage".to_owned(),
        ],
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

fn pushout_groups(left: &BTreeMap<Id, Id>, right: &BTreeMap<Id, Id>) -> PushoutGroups {
    let left_source_ids = left.keys().cloned().collect::<BTreeSet<_>>();
    let right_source_ids = right.keys().cloned().collect::<BTreeSet<_>>();
    let identified_groups = left_source_ids
        .intersection(&right_source_ids)
        .map(|source_element_id| IdentifiedSourceGroup {
            source_element_id: source_element_id.clone(),
            left_target_id: left[source_element_id].clone(),
            right_target_id: right[source_element_id].clone(),
        })
        .collect();

    PushoutGroups {
        identified_groups,
        unmatched_left_source_ids: left_source_ids
            .difference(&right_source_ids)
            .cloned()
            .collect(),
        unmatched_right_source_ids: right_source_ids
            .difference(&left_source_ids)
            .cloned()
            .collect(),
    }
}

fn compose_path_summary(path: &[Morphism]) -> DiagramPathSummary {
    let morphism_ids = path.iter().map(|morphism| morphism.id.clone()).collect();
    let Some(first) = path.first() else {
        return DiagramPathSummary {
            morphism_ids,
            source_space_id: None,
            target_space_id: None,
            cell_mapping: BTreeMap::new(),
            relation_mapping: BTreeMap::new(),
            coverage: CompositionCoverage::default(),
        };
    };

    let mut cell_mapping = first.cell_mapping.clone();
    let mut relation_mapping = first.relation_mapping.clone();
    let mut unmapped_cell_intermediate_ids = BTreeSet::new();
    let mut unmapped_relation_intermediate_ids = BTreeSet::new();

    for next in path.iter().skip(1) {
        let cell_composition = compose_mapping_parts(&cell_mapping, &next.cell_mapping);
        let relation_composition = compose_mapping_parts(&relation_mapping, &next.relation_mapping);
        unmapped_cell_intermediate_ids.extend(unmapped_intermediate_ids(&cell_composition));
        unmapped_relation_intermediate_ids.extend(unmapped_intermediate_ids(&relation_composition));
        cell_mapping = cell_composition.mapping;
        relation_mapping = relation_composition.mapping;
    }

    DiagramPathSummary {
        morphism_ids,
        source_space_id: Some(first.source_space_id.clone()),
        target_space_id: path.last().map(|morphism| morphism.target_space_id.clone()),
        cell_mapping,
        relation_mapping,
        coverage: CompositionCoverage {
            unmapped_cell_intermediate_ids: unmapped_cell_intermediate_ids.into_iter().collect(),
            unmapped_relation_intermediate_ids: unmapped_relation_intermediate_ids
                .into_iter()
                .collect(),
        },
    }
}

fn path_obstructions(label: &str, path: &DiagramPathSummary) -> Vec<DiagramObstruction> {
    let mut obstructions = Vec::new();
    if path.morphism_ids.is_empty() {
        obstructions.push(DiagramObstruction {
            obstruction_type: DiagramObstructionType::IncompatiblePath,
            reason: format!("{label} path is empty"),
        });
    }
    if !path.coverage.is_complete() {
        obstructions.push(DiagramObstruction {
            obstruction_type: DiagramObstructionType::IncompletePath,
            reason: format!("{label} path has explicit mappings that cannot continue"),
        });
    }
    obstructions
}

fn path_boundary_obstructions(label: &str, path: &[Morphism]) -> Vec<DiagramObstruction> {
    path.windows(2)
        .filter_map(|window| {
            let first = &window[0];
            let second = &window[1];
            (first.target_space_id != second.source_space_id).then(|| DiagramObstruction {
                obstruction_type: DiagramObstructionType::IncompatiblePath,
                reason: format!(
                    "{label} path morphism {} targets {}, but next morphism {} sources {}",
                    first.id, first.target_space_id, second.id, second.source_space_id
                ),
            })
        })
        .collect()
}

fn mapping_witnesses(
    element_kind: DiagramElementKind,
    left: &BTreeMap<Id, Id>,
    right: &BTreeMap<Id, Id>,
) -> Vec<NonCommutativeWitness> {
    let source_ids = left
        .keys()
        .chain(right.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    source_ids
        .into_iter()
        .filter_map(|source_element_id| {
            let left_target_id = left.get(&source_element_id);
            let right_target_id = right.get(&source_element_id);
            (left_target_id != right_target_id).then(|| NonCommutativeWitness {
                element_kind,
                source_element_id,
                left_target_id: left_target_id.cloned(),
                right_target_id: right_target_id.cloned(),
            })
        })
        .collect()
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

#[derive(Debug)]
struct PushoutGroups {
    identified_groups: Vec<IdentifiedSourceGroup>,
    unmatched_left_source_ids: Vec<Id>,
    unmatched_right_source_ids: Vec<Id>,
}

impl PushoutGroups {
    fn into_parts(self) -> (Vec<IdentifiedSourceGroup>, Vec<Id>, Vec<Id>) {
        (
            self.identified_groups,
            self.unmatched_left_source_ids,
            self.unmatched_right_source_ids,
        )
    }
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
