//! Structure mappings, composition, preservation checks, lost structure, and
//! distortion for HigherGraphen.

use higher_graphen_core::{Id, Provenance, ReviewStatus, Severity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::space::{Cell, Complex, ComplexType, Incidence, Space};

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
    /// Matched source cells or relations disagree on attributes required for one fiber element.
    IncompatibleFiber,
}

impl PullbackObstructionType {
    /// Returns true when this obstruction prevents materializing the pullback candidate.
    #[must_use]
    pub fn is_blocking(&self) -> bool {
        true
    }
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

/// Explicit finite inputs used to construct a binary pullback candidate.
#[derive(Clone, Debug)]
pub struct PullbackInputs {
    /// Left cospan leg from its source space into the shared target space.
    pub left: Morphism,
    /// Right cospan leg from its source space into the shared target space.
    pub right: Morphism,
    /// Identifier assigned to the materialized candidate space.
    pub candidate_space_id: Id,
    /// Human-readable name assigned to the materialized candidate space and complex.
    pub candidate_space_name: String,
    /// Structural kind assigned to the materialized candidate complex.
    pub complex_type: ComplexType,
    /// Cells from the left source space to pair in the finite fiber product.
    pub left_source_cells: Vec<Cell>,
    /// Cells from the right source space to pair in the finite fiber product.
    pub right_source_cells: Vec<Cell>,
    /// Incidences from the left source space to pair in the finite fiber product.
    pub left_source_incidences: Vec<Incidence>,
    /// Incidences from the right source space to pair in the finite fiber product.
    pub right_source_incidences: Vec<Incidence>,
}

/// Materialized finite pullback candidate.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PullbackConstruction {
    /// Populated candidate space containing exactly the constructed memberships.
    pub space: Space,
    /// Candidate complex over the constructed cells and incidences.
    pub complex: Complex,
    /// Constructed fiber cells sorted by identifier.
    pub cells: Vec<Cell>,
    /// Constructed fiber incidences sorted by identifier.
    pub incidences: Vec<Incidence>,
    /// Cell-pair matches used to construct the fiber cells.
    pub cell_matches: Vec<PullbackCellMatch>,
    /// Relation-pair matches used to construct the fiber incidences.
    pub relation_matches: Vec<PullbackRelationMatch>,
}

/// Result of constructing a finite explicit pullback.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PullbackOutcome {
    /// The pullback was materialized as a reviewable candidate.
    Constructed {
        /// The materialized candidate structure.
        construction: Box<PullbackConstruction>,
        /// Diagnostic report for the construction.
        report: ExplicitPullbackReport,
    },
    /// Blocking obstructions prevented materializing a valid candidate.
    Blocked {
        /// Diagnostic report carrying all detected obstructions.
        report: ExplicitPullbackReport,
    },
}

/// Stable obstruction emitted by explicit pushout-candidate extraction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PushoutObstructionType {
    /// The two morphisms do not start from the same source space.
    IncompatibleSourceSpace,
    /// At least one source mapping has no partner on the other leg.
    PushoutIncomplete,
    /// Identified cells or relations disagree on attributes required to merge them.
    IncompatibleIdentification,
    /// Identified relations do not agree on remapped endpoints.
    RelationEndpointConflict,
    /// The quotient collapses two or more originally distinct same-side elements.
    AmbiguousIdentification,
}

impl PushoutObstructionType {
    /// Returns true when this obstruction prevents materializing the pushout candidate.
    #[must_use]
    pub fn is_blocking(&self) -> bool {
        !matches!(self, Self::AmbiguousIdentification)
    }
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
    /// Returns true when no pushout obstructions were detected.
    pub fn is_complete(&self) -> bool {
        self.obstructions.is_empty()
    }
}

/// Explicit finite inputs used to construct a binary pushout candidate.
pub struct PushoutInputs<'a> {
    /// Left cospan leg from the shared source into the left target space.
    pub left: &'a Morphism,
    /// Right cospan leg from the shared source into the right target space.
    pub right: &'a Morphism,
    /// Identifier assigned to the materialized candidate space.
    pub candidate_space_id: Id,
    /// Human-readable name assigned to the materialized candidate space and complex.
    pub candidate_space_name: String,
    /// Structural kind assigned to the materialized candidate complex.
    pub complex_type: ComplexType,
    /// Cells from the left target space to carry into the finite quotient.
    pub left_cells: &'a [Cell],
    /// Cells from the right target space to carry into the finite quotient.
    pub right_cells: &'a [Cell],
    /// Incidences from the left target space to carry into the finite quotient.
    pub left_incidences: &'a [Incidence],
    /// Incidences from the right target space to carry into the finite quotient.
    pub right_incidences: &'a [Incidence],
}

/// Materialized finite pushout candidate.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PushoutConstruction {
    /// Populated candidate space containing exactly the constructed memberships.
    pub space: Space,
    /// Candidate complex over the constructed cells and incidences.
    pub complex: Complex,
    /// Constructed cells sorted by identifier.
    pub cells: Vec<Cell>,
    /// Constructed incidences sorted by identifier.
    pub incidences: Vec<Incidence>,
    /// Review status for the materialized candidate; this is never accepted.
    pub review_status: ReviewStatus,
}

/// Result of constructing a finite explicit pushout.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PushoutOutcome {
    /// The pushout was materialized as a reviewable candidate.
    Constructed {
        /// The materialized candidate structure.
        construction: Box<PushoutConstruction>,
        /// Diagnostic report for the construction.
        report: ExplicitPushoutReport,
    },
    /// Blocking obstructions prevented materializing a valid candidate.
    Blocked {
        /// Diagnostic report carrying all detected obstructions.
        report: ExplicitPushoutReport,
    },
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

/// One finite commutativity requirement in a diagram.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagramCommutativityRequirement {
    /// Stable requirement identifier.
    pub id: Id,
    /// Left morphism path.
    pub left_path: Vec<Morphism>,
    /// Right morphism path.
    pub right_path: Vec<Morphism>,
}

impl DiagramCommutativityRequirement {
    /// Creates a two-path commutativity requirement.
    #[must_use]
    pub fn new(id: Id, left_path: Vec<Morphism>, right_path: Vec<Morphism>) -> Self {
        Self {
            id,
            left_path,
            right_path,
        }
    }
}

/// Report for a finite diagram with multiple commutativity requirements.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagramCheckReport {
    /// Stable diagram identifier supplied by the caller.
    pub diagram_id: Id,
    /// True when every requirement commutes.
    pub commutes: bool,
    /// Per-requirement commutativity reports.
    pub requirement_reports: Vec<DiagramRequirementReport>,
}

/// Per-requirement diagram check result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagramRequirementReport {
    /// Requirement checked.
    pub requirement_id: Id,
    /// Two-path commutativity report.
    pub report: DiagramCommutativityReport,
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
