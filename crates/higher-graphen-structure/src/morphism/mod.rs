//! Structure mappings, composition, preservation checks, lost structure, and
//! distortion for HigherGraphen.

use higher_graphen_core::{Id, Provenance, ReviewStatus, Severity};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::space::{Cell, Complex, ComplexType, Incidence, IncidenceOrientation, Space};

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
        /// Blocking obstructions collected during construction.
        obstructions: Vec<PullbackObstruction>,
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

/// Constructs a deterministic finite pushout candidate for a binary cospan.
///
/// The construction quotients the disjoint union of the left and right target
/// cells and incidences by the identifications induced from shared source
/// mappings. Blocking obstructions return [`PushoutOutcome::Blocked`] and do
/// not materialize cells, incidences, or a complex.
pub fn construct_explicit_pushout(inputs: PushoutInputs<'_>) -> PushoutOutcome {
    let mut report =
        explicit_pushout_candidate(inputs.left, inputs.right, inputs.candidate_space_id.clone());
    let mut quotient_losses = Vec::new();

    let left_cells = inputs
        .left_cells
        .iter()
        .map(|cell| (cell.id.clone(), cell))
        .collect::<BTreeMap<_, _>>();
    let right_cells = inputs
        .right_cells
        .iter()
        .map(|cell| (cell.id.clone(), cell))
        .collect::<BTreeMap<_, _>>();

    let mut cell_union = PushoutUnionFind::default();
    for cell in inputs.left_cells {
        cell_union.insert(PushoutElementKey::left(cell.id.clone()));
    }
    for cell in inputs.right_cells {
        cell_union.insert(PushoutElementKey::right(cell.id.clone()));
    }
    for group in &report.identified_cell_groups {
        cell_union.union(
            PushoutElementKey::left(group.left_target_id.clone()),
            PushoutElementKey::right(group.right_target_id.clone()),
        );
    }

    let Some(cell_classes) = cell_union.classes(&inputs.candidate_space_id, "cell") else {
        report.obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::IncompatibleIdentification,
            reason: "could not derive a valid canonical cell identifier".to_owned(),
        });
        report.quotient_losses = quotient_losses;
        report.review_status = pushout_review_status(&report.obstructions);
        return PushoutOutcome::Blocked { report };
    };

    let mut cell_id_by_key = BTreeMap::new();
    for class in &cell_classes {
        for member in &class.members {
            cell_id_by_key.insert(member.clone(), class.canonical_id.clone());
        }
    }

    report.obstructions.extend(ambiguous_class_obstructions(
        "cell",
        &cell_classes,
        &mut quotient_losses,
    ));

    let mut cells = Vec::new();
    for class in &cell_classes {
        if let Some(cell) = merged_cell(
            class,
            &inputs.candidate_space_id,
            &left_cells,
            &right_cells,
            &cell_id_by_key,
            &mut report.obstructions,
            &mut quotient_losses,
        ) {
            cells.push(cell);
        }
    }
    cells.sort_by(|left, right| left.id.cmp(&right.id));

    let left_incidences = inputs
        .left_incidences
        .iter()
        .map(|incidence| (incidence.id.clone(), incidence))
        .collect::<BTreeMap<_, _>>();
    let right_incidences = inputs
        .right_incidences
        .iter()
        .map(|incidence| (incidence.id.clone(), incidence))
        .collect::<BTreeMap<_, _>>();

    let mut relation_union = PushoutUnionFind::default();
    for incidence in inputs.left_incidences {
        relation_union.insert(PushoutElementKey::left(incidence.id.clone()));
    }
    for incidence in inputs.right_incidences {
        relation_union.insert(PushoutElementKey::right(incidence.id.clone()));
    }
    for group in &report.identified_relation_groups {
        relation_union.union(
            PushoutElementKey::left(group.left_target_id.clone()),
            PushoutElementKey::right(group.right_target_id.clone()),
        );
    }

    let Some(relation_classes) = relation_union.classes(&inputs.candidate_space_id, "incidence")
    else {
        report.obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::IncompatibleIdentification,
            reason: "could not derive a valid canonical incidence identifier".to_owned(),
        });
        report.quotient_losses = quotient_losses;
        report.review_status = pushout_review_status(&report.obstructions);
        return PushoutOutcome::Blocked { report };
    };

    report.obstructions.extend(ambiguous_class_obstructions(
        "relation",
        &relation_classes,
        &mut quotient_losses,
    ));

    let incidence_seeds = relation_classes
        .iter()
        .filter_map(|class| {
            merged_incidence_seed(
                class,
                &inputs.candidate_space_id,
                &left_incidences,
                &right_incidences,
                &cell_id_by_key,
                &mut report.obstructions,
                &mut quotient_losses,
            )
        })
        .collect::<Vec<_>>();
    let mut incidences = deduplicate_incidences(incidence_seeds, &mut quotient_losses);
    incidences.sort_by(|left, right| left.id.cmp(&right.id));

    report.quotient_losses = quotient_losses;
    report.review_status = pushout_review_status(&report.obstructions);

    if report
        .obstructions
        .iter()
        .any(|obstruction| obstruction.obstruction_type.is_blocking())
    {
        return PushoutOutcome::Blocked { report };
    }

    let review_status = report.review_status;
    let (mut space, mut complex) = assemble_candidate(
        &inputs.candidate_space_id,
        "pushout",
        inputs.complex_type,
        &cells,
        &incidences,
    );
    space.name = inputs.candidate_space_name.trim().to_owned();
    complex.name = inputs.candidate_space_name.trim().to_owned();

    PushoutOutcome::Constructed {
        construction: Box::new(PushoutConstruction {
            space,
            complex,
            cells,
            incidences,
            review_status,
        }),
        report,
    }
}

/// Constructs a deterministic finite pullback candidate for a binary cospan.
///
/// The construction pairs left and right source cells whose morphism mappings
/// agree in the shared target. Blocking obstructions return
/// [`PullbackOutcome::Blocked`] and do not materialize cells, incidences, or a
/// complex.
pub fn construct_explicit_pullback(inputs: PullbackInputs) -> PullbackOutcome {
    let mut report = explicit_pullback_candidate(&inputs.left, &inputs.right);

    if inputs.left.target_space_id != inputs.right.target_space_id {
        report.review_status = ReviewStatus::Rejected;
        return PullbackOutcome::Blocked {
            obstructions: report.obstructions.clone(),
            report,
        };
    }

    let left_cells = inputs
        .left_source_cells
        .iter()
        .map(|cell| (cell.id.clone(), cell))
        .collect::<BTreeMap<_, _>>();
    let right_cells = inputs
        .right_source_cells
        .iter()
        .map(|cell| (cell.id.clone(), cell))
        .collect::<BTreeMap<_, _>>();
    let left_incidences = inputs
        .left_source_incidences
        .iter()
        .map(|incidence| (incidence.id.clone(), incidence))
        .collect::<BTreeMap<_, _>>();
    let right_incidences = inputs
        .right_source_incidences
        .iter()
        .map(|incidence| (incidence.id.clone(), incidence))
        .collect::<BTreeMap<_, _>>();

    let mut information_loss = report.information_loss.clone();
    let mut cell_id_by_pair = BTreeMap::<(Id, Id), Id>::new();
    let mut cells = Vec::new();

    for matched in &report.cell_matches {
        let Some(canonical_id) = pullback_cell_id(
            &inputs.candidate_space_id,
            &matched.left_cell_id,
            &matched.right_cell_id,
        ) else {
            report.obstructions.push(PullbackObstruction {
                obstruction_type: PullbackObstructionType::IncompatibleFiber,
                reason: format!(
                    "could not derive a valid canonical cell identifier for pair ({}, {})",
                    matched.left_cell_id, matched.right_cell_id
                ),
            });
            continue;
        };
        cell_id_by_pair.insert(
            (matched.left_cell_id.clone(), matched.right_cell_id.clone()),
            canonical_id,
        );
    }

    for matched in &report.cell_matches {
        let Some(canonical_id) =
            cell_id_by_pair.get(&(matched.left_cell_id.clone(), matched.right_cell_id.clone()))
        else {
            continue;
        };
        let (Some(left_cell), Some(right_cell)) = (
            left_cells.get(&matched.left_cell_id),
            right_cells.get(&matched.right_cell_id),
        ) else {
            report.obstructions.push(PullbackObstruction {
                obstruction_type: PullbackObstructionType::IncompatibleFiber,
                reason: format!(
                    "matched cell pair ({}, {}) is not present in the finite source inputs",
                    matched.left_cell_id, matched.right_cell_id
                ),
            });
            continue;
        };

        if left_cell.dimension != right_cell.dimension
            || left_cell.cell_type != right_cell.cell_type
        {
            report.obstructions.push(PullbackObstruction {
                obstruction_type: PullbackObstructionType::IncompatibleFiber,
                reason: format!(
                    "matched cell pair ({}, {}) has incompatible dimensions {} vs {} or cell types {:?} vs {:?}",
                    matched.left_cell_id,
                    matched.right_cell_id,
                    left_cell.dimension,
                    right_cell.dimension,
                    left_cell.cell_type,
                    right_cell.cell_type
                ),
            });
            continue;
        }

        let label = [left_cell.label.clone(), right_cell.label.clone()]
            .into_iter()
            .flatten()
            .min();
        let mut boundary = BTreeSet::new();
        for left_boundary_id in &left_cell.boundary {
            for right_boundary_id in &right_cell.boundary {
                if let Some(boundary_id) =
                    cell_id_by_pair.get(&(left_boundary_id.clone(), right_boundary_id.clone()))
                {
                    boundary.insert(boundary_id.clone());
                }
            }
        }
        let mut coboundary = BTreeSet::new();
        for left_coboundary_id in &left_cell.coboundary {
            for right_coboundary_id in &right_cell.coboundary {
                if let Some(coboundary_id) =
                    cell_id_by_pair.get(&(left_coboundary_id.clone(), right_coboundary_id.clone()))
                {
                    coboundary.insert(coboundary_id.clone());
                }
            }
        }
        let context_ids = left_cell
            .context_ids
            .iter()
            .chain(right_cell.context_ids.iter())
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if left_cell.provenance.is_some() || right_cell.provenance.is_some() {
            information_loss.push(format!(
                "cell pair ({}, {}) provenance dropped because the pullback cell has two sources",
                matched.left_cell_id, matched.right_cell_id
            ));
        }

        cells.push(Cell {
            id: canonical_id.clone(),
            space_id: inputs.candidate_space_id.clone(),
            dimension: left_cell.dimension,
            cell_type: left_cell.cell_type.clone(),
            label,
            boundary: boundary.into_iter().collect(),
            coboundary: coboundary.into_iter().collect(),
            context_ids,
            provenance: None,
        });
    }
    cells.sort_by(|left, right| left.id.cmp(&right.id));

    let mut incidence_seeds = Vec::new();
    for matched in &report.relation_matches {
        let Some(canonical_id) = pullback_incidence_id(
            &inputs.candidate_space_id,
            &matched.left_relation_id,
            &matched.right_relation_id,
        ) else {
            report.obstructions.push(PullbackObstruction {
                obstruction_type: PullbackObstructionType::IncompatibleFiber,
                reason: format!(
                    "could not derive a valid canonical incidence identifier for pair ({}, {})",
                    matched.left_relation_id, matched.right_relation_id
                ),
            });
            continue;
        };

        let (Some(left_incidence), Some(right_incidence)) = (
            left_incidences.get(&matched.left_relation_id),
            right_incidences.get(&matched.right_relation_id),
        ) else {
            report.obstructions.push(PullbackObstruction {
                obstruction_type: PullbackObstructionType::IncompatibleFiber,
                reason: format!(
                    "matched relation pair ({}, {}) is not present in the finite source inputs",
                    matched.left_relation_id, matched.right_relation_id
                ),
            });
            continue;
        };

        if left_incidence.relation_type != right_incidence.relation_type
            || left_incidence.orientation != right_incidence.orientation
        {
            report.obstructions.push(PullbackObstruction {
                obstruction_type: PullbackObstructionType::IncompatibleFiber,
                reason: format!(
                    "matched relation pair ({}, {}) has incompatible relation types {:?} vs {:?} or orientations {:?} vs {:?}",
                    matched.left_relation_id,
                    matched.right_relation_id,
                    left_incidence.relation_type,
                    right_incidence.relation_type,
                    left_incidence.orientation,
                    right_incidence.orientation
                ),
            });
            continue;
        }

        let from_pair = (
            left_incidence.from_cell_id.clone(),
            right_incidence.from_cell_id.clone(),
        );
        let to_pair = (
            left_incidence.to_cell_id.clone(),
            right_incidence.to_cell_id.clone(),
        );
        let (Some(from_cell_id), Some(to_cell_id)) = (
            cell_id_by_pair.get(&from_pair),
            cell_id_by_pair.get(&to_pair),
        ) else {
            information_loss.push(format!(
                "relation pair ({}, {}) dropped because one or both endpoint pairs are outside the pullback cells",
                matched.left_relation_id, matched.right_relation_id
            ));
            continue;
        };

        if left_incidence.provenance.is_some() || right_incidence.provenance.is_some() {
            information_loss.push(format!(
                "incidence pair ({}, {}) provenance dropped because the pullback incidence has two sources",
                matched.left_relation_id, matched.right_relation_id
            ));
        }

        incidence_seeds.push(IncidenceSeed {
            signature: IncidenceSignature {
                from_cell_id: from_cell_id.clone(),
                to_cell_id: to_cell_id.clone(),
                relation_type: left_incidence.relation_type.clone(),
                orientation: left_incidence.orientation,
            },
            incidence: Incidence {
                id: canonical_id,
                space_id: inputs.candidate_space_id.clone(),
                from_cell_id: from_cell_id.clone(),
                to_cell_id: to_cell_id.clone(),
                relation_type: left_incidence.relation_type.clone(),
                orientation: left_incidence.orientation,
                weight: left_incidence.weight,
                provenance: None,
            },
        });
    }
    let mut incidences = deduplicate_incidences(incidence_seeds, &mut information_loss);
    incidences.sort_by(|left, right| left.id.cmp(&right.id));

    report.information_loss = information_loss;
    report.review_status = if report
        .obstructions
        .iter()
        .any(|obstruction| obstruction.obstruction_type.is_blocking())
    {
        ReviewStatus::Rejected
    } else {
        ReviewStatus::Candidate
    };

    if report
        .obstructions
        .iter()
        .any(|obstruction| obstruction.obstruction_type.is_blocking())
    {
        return PullbackOutcome::Blocked {
            obstructions: report.obstructions.clone(),
            report,
        };
    }

    let (mut space, mut complex) = assemble_candidate(
        &inputs.candidate_space_id,
        "pullback",
        inputs.complex_type,
        &cells,
        &incidences,
    );
    space.name = inputs.candidate_space_name.trim().to_owned();
    complex.name = inputs.candidate_space_name.trim().to_owned();

    PullbackOutcome::Constructed {
        construction: Box::new(PullbackConstruction {
            space,
            complex,
            cells,
            incidences,
            cell_matches: report.cell_matches.clone(),
            relation_matches: report.relation_matches.clone(),
        }),
        report,
    }
}

fn pullback_cell_id(candidate_space_id: &Id, left_cell_id: &Id, right_cell_id: &Id) -> Option<Id> {
    Id::new(format!(
        "{}/pullback/cell/{}+{}",
        candidate_space_id.as_str(),
        encode_pushout_fragment(left_cell_id.as_str()),
        encode_pushout_fragment(right_cell_id.as_str())
    ))
    .ok()
}

fn pullback_incidence_id(
    candidate_space_id: &Id,
    left_relation_id: &Id,
    right_relation_id: &Id,
) -> Option<Id> {
    Id::new(format!(
        "{}/pullback/incidence/{}+{}",
        candidate_space_id.as_str(),
        encode_pushout_fragment(left_relation_id.as_str()),
        encode_pushout_fragment(right_relation_id.as_str())
    ))
    .ok()
}

fn pushout_review_status(obstructions: &[PushoutObstruction]) -> ReviewStatus {
    if obstructions
        .iter()
        .any(|obstruction| obstruction.obstruction_type.is_blocking())
    {
        ReviewStatus::Rejected
    } else if obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == PushoutObstructionType::AmbiguousIdentification
    }) {
        ReviewStatus::Candidate
    } else {
        ReviewStatus::Unreviewed
    }
}

fn class_member_cell<'a>(
    key: &PushoutElementKey,
    left_cells: &'a BTreeMap<Id, &'a Cell>,
    right_cells: &'a BTreeMap<Id, &'a Cell>,
) -> Option<&'a Cell> {
    match key.side {
        PushoutSide::Left => left_cells.get(&key.id).copied(),
        PushoutSide::Right => right_cells.get(&key.id).copied(),
    }
}

fn class_member_incidence<'a>(
    key: &PushoutElementKey,
    left_incidences: &'a BTreeMap<Id, &'a Incidence>,
    right_incidences: &'a BTreeMap<Id, &'a Incidence>,
) -> Option<&'a Incidence> {
    match key.side {
        PushoutSide::Left => left_incidences.get(&key.id).copied(),
        PushoutSide::Right => right_incidences.get(&key.id).copied(),
    }
}

fn ambiguous_class_obstructions(
    element_kind: &str,
    classes: &[PushoutEquivalenceClass],
    quotient_losses: &mut Vec<String>,
) -> Vec<PushoutObstruction> {
    let mut obstructions = Vec::new();
    for class in classes {
        let left_ids = class
            .members
            .iter()
            .filter(|member| member.side == PushoutSide::Left)
            .map(|member| member.id.clone())
            .collect::<Vec<_>>();
        let right_ids = class
            .members
            .iter()
            .filter(|member| member.side == PushoutSide::Right)
            .map(|member| member.id.clone())
            .collect::<Vec<_>>();

        if left_ids.len() > 1 || right_ids.len() > 1 {
            obstructions.push(PushoutObstruction {
                obstruction_type: PushoutObstructionType::AmbiguousIdentification,
                reason: format!(
                    "{element_kind} class {} collapses multiple same-side elements",
                    class.canonical_id
                ),
            });
            quotient_losses.push(format!(
                "ambiguous_identification: {element_kind} class {} collapses left {:?} and right {:?}",
                class.canonical_id, left_ids, right_ids
            ));
        }
    }
    obstructions
}

fn remap_cell_reference(
    side: &PushoutSide,
    cell_id: &Id,
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
) -> Option<Id> {
    let key = match side {
        PushoutSide::Left => PushoutElementKey::left(cell_id.clone()),
        PushoutSide::Right => PushoutElementKey::right(cell_id.clone()),
    };
    cell_id_by_key.get(&key).cloned()
}

fn remap_cell_refs(
    owner: &PushoutElementKey,
    attribute: &str,
    refs: &[Id],
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
    quotient_losses: &mut Vec<String>,
) -> BTreeSet<Id> {
    let mut remapped = BTreeSet::new();
    for cell_id in refs {
        if let Some(mapped_id) = remap_cell_reference(&owner.side, cell_id, cell_id_by_key) {
            remapped.insert(mapped_id);
        } else {
            quotient_losses.push(format!(
                "cell {}:{} {attribute} reference {} dropped because it is outside the pushout input",
                owner.side.as_str(),
                owner.id,
                cell_id
            ));
        }
    }
    remapped
}

fn merged_cell(
    class: &PushoutEquivalenceClass,
    candidate_space_id: &Id,
    left_cells: &BTreeMap<Id, &Cell>,
    right_cells: &BTreeMap<Id, &Cell>,
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
    obstructions: &mut Vec<PushoutObstruction>,
    quotient_losses: &mut Vec<String>,
) -> Option<Cell> {
    let mut members = Vec::new();
    for key in &class.members {
        if let Some(cell) = class_member_cell(key, left_cells, right_cells) {
            members.push((key, cell));
        } else {
            obstructions.push(PushoutObstruction {
                obstruction_type: PushoutObstructionType::IncompatibleIdentification,
                reason: format!(
                    "identified cell {}:{} is not present in the finite input",
                    key.side.as_str(),
                    key.id
                ),
            });
        }
    }

    let dimensions = members
        .iter()
        .map(|(_, cell)| cell.dimension)
        .collect::<BTreeSet<_>>();
    let cell_types = members
        .iter()
        .map(|(_, cell)| cell.cell_type.clone())
        .collect::<BTreeSet<_>>();

    if dimensions.len() > 1 || cell_types.len() > 1 {
        obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::IncompatibleIdentification,
            reason: format!(
                "cell class {} has incompatible dimensions {:?} or cell types {:?}",
                class.canonical_id, dimensions, cell_types
            ),
        });
        return None;
    }

    let dimension = dimensions.iter().next().copied()?;
    let cell_type = cell_types.iter().next().cloned()?;

    let canonical_label = members
        .iter()
        .filter_map(|(_, cell)| cell.label.clone())
        .min();
    let distinct_labels = members
        .iter()
        .map(|(_, cell)| cell.label.clone())
        .collect::<BTreeSet<_>>();
    if distinct_labels.len() > 1 {
        for (key, cell) in &members {
            if cell.label != canonical_label {
                quotient_losses.push(format!(
                    "cell {}:{} label {:?} dropped; class {} keeps {:?}",
                    key.side.as_str(),
                    key.id,
                    cell.label,
                    class.canonical_id,
                    canonical_label
                ));
            }
        }
    }

    let mut boundary = BTreeSet::new();
    let mut coboundary = BTreeSet::new();
    let mut context_ids = BTreeSet::new();
    for (key, cell) in &members {
        boundary.extend(remap_cell_refs(
            key,
            "boundary",
            &cell.boundary,
            cell_id_by_key,
            quotient_losses,
        ));
        coboundary.extend(remap_cell_refs(
            key,
            "coboundary",
            &cell.coboundary,
            cell_id_by_key,
            quotient_losses,
        ));
        context_ids.extend(cell.context_ids.iter().cloned());
    }

    let provenance = if members.len() == 1 {
        members
            .first()
            .and_then(|(_, cell)| cell.provenance.clone())
    } else {
        for (key, cell) in &members {
            if cell.provenance.is_some() {
                quotient_losses.push(format!(
                    "cell {}:{} provenance dropped; merged class {} has multiple sources",
                    key.side.as_str(),
                    key.id,
                    class.canonical_id
                ));
            }
        }
        None
    };

    Some(Cell {
        id: class.canonical_id.clone(),
        space_id: candidate_space_id.clone(),
        dimension,
        cell_type,
        label: canonical_label,
        boundary: boundary.into_iter().collect(),
        coboundary: coboundary.into_iter().collect(),
        context_ids: context_ids.into_iter().collect(),
        provenance,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct IncidenceSignature {
    from_cell_id: Id,
    to_cell_id: Id,
    relation_type: String,
    orientation: IncidenceOrientation,
}

#[derive(Clone, Debug)]
struct IncidenceSeed {
    incidence: Incidence,
    signature: IncidenceSignature,
}

fn merged_incidence_seed(
    class: &PushoutEquivalenceClass,
    candidate_space_id: &Id,
    left_incidences: &BTreeMap<Id, &Incidence>,
    right_incidences: &BTreeMap<Id, &Incidence>,
    cell_id_by_key: &BTreeMap<PushoutElementKey, Id>,
    obstructions: &mut Vec<PushoutObstruction>,
    quotient_losses: &mut Vec<String>,
) -> Option<IncidenceSeed> {
    let mut members = Vec::new();
    for key in &class.members {
        if let Some(incidence) = class_member_incidence(key, left_incidences, right_incidences) {
            let from_cell_id =
                remap_cell_reference(&key.side, &incidence.from_cell_id, cell_id_by_key);
            let to_cell_id = remap_cell_reference(&key.side, &incidence.to_cell_id, cell_id_by_key);
            if from_cell_id.is_none() || to_cell_id.is_none() {
                obstructions.push(PushoutObstruction {
                    obstruction_type: PushoutObstructionType::RelationEndpointConflict,
                    reason: format!(
                        "incidence {}:{} references an endpoint outside the pushout cells",
                        key.side.as_str(),
                        key.id
                    ),
                });
            }
            if let (Some(from_cell_id), Some(to_cell_id)) = (from_cell_id, to_cell_id) {
                members.push((key, incidence, from_cell_id, to_cell_id));
            }
        } else {
            obstructions.push(PushoutObstruction {
                obstruction_type: PushoutObstructionType::IncompatibleIdentification,
                reason: format!(
                    "identified relation {}:{} is not present in the finite input",
                    key.side.as_str(),
                    key.id
                ),
            });
        }
    }

    let relation_types = members
        .iter()
        .map(|(_, incidence, _, _)| incidence.relation_type.clone())
        .collect::<BTreeSet<_>>();
    let orientations = members
        .iter()
        .map(|(_, incidence, _, _)| incidence.orientation)
        .collect::<BTreeSet<_>>();
    if relation_types.len() > 1 || orientations.len() > 1 {
        obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::IncompatibleIdentification,
            reason: format!(
                "relation class {} has incompatible relation types {:?} or orientations {:?}",
                class.canonical_id, relation_types, orientations
            ),
        });
        return None;
    }

    let endpoints = members
        .iter()
        .map(|(_, _, from_cell_id, to_cell_id)| (from_cell_id.clone(), to_cell_id.clone()))
        .collect::<BTreeSet<_>>();
    if endpoints.len() > 1 {
        obstructions.push(PushoutObstruction {
            obstruction_type: PushoutObstructionType::RelationEndpointConflict,
            reason: format!(
                "relation class {} has conflicting remapped endpoints {:?}",
                class.canonical_id, endpoints
            ),
        });
        return None;
    }

    let (_, first, from_cell_id, to_cell_id) = members.first()?;
    let relation_type = relation_types.iter().next().cloned()?;
    let orientation = orientations.iter().next().copied()?;

    let weight = first.weight;
    let provenance = if members.len() == 1 {
        first.provenance.clone()
    } else {
        for (key, incidence, _, _) in &members {
            if incidence.weight != weight {
                quotient_losses.push(format!(
                    "incidence {}:{} weight {:?} dropped; class {} keeps {:?}",
                    key.side.as_str(),
                    key.id,
                    incidence.weight,
                    class.canonical_id,
                    weight
                ));
            }
            if incidence.provenance.is_some() {
                quotient_losses.push(format!(
                    "incidence {}:{} provenance dropped; merged class {} has multiple sources",
                    key.side.as_str(),
                    key.id,
                    class.canonical_id
                ));
            }
        }
        None
    };
    let signature = IncidenceSignature {
        from_cell_id: from_cell_id.clone(),
        to_cell_id: to_cell_id.clone(),
        relation_type,
        orientation,
    };

    Some(IncidenceSeed {
        incidence: Incidence {
            id: class.canonical_id.clone(),
            space_id: candidate_space_id.clone(),
            from_cell_id: signature.from_cell_id.clone(),
            to_cell_id: signature.to_cell_id.clone(),
            relation_type: signature.relation_type.clone(),
            orientation: signature.orientation,
            weight,
            provenance,
        },
        signature,
    })
}

fn deduplicate_incidences(
    mut seeds: Vec<IncidenceSeed>,
    quotient_losses: &mut Vec<String>,
) -> Vec<Incidence> {
    seeds.sort_by(|left, right| left.incidence.id.cmp(&right.incidence.id));
    let mut by_signature = BTreeMap::<IncidenceSignature, Incidence>::new();
    for seed in seeds {
        if let Some(existing) = by_signature.get(&seed.signature) {
            if existing.weight != seed.incidence.weight {
                quotient_losses.push(format!(
                    "incidence {} weight {:?} dropped during dedup; incidence {} keeps {:?}",
                    seed.incidence.id, seed.incidence.weight, existing.id, existing.weight
                ));
            }
            if existing.provenance != seed.incidence.provenance {
                quotient_losses.push(format!(
                    "incidence {} provenance dropped during dedup; incidence {} is canonical",
                    seed.incidence.id, existing.id
                ));
            }
        } else {
            by_signature.insert(seed.signature, seed.incidence);
        }
    }
    by_signature.into_values().collect()
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

/// Extracts an explicit pushout-style merge report for two morphisms sharing a source.
///
/// The candidate identifies left and right targets that come from the same
/// source element. Use [`construct_explicit_pushout`] with finite cells and
/// incidences to materialize the merged candidate structure.
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
        quotient_losses: Vec::new(),
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

/// Checks multiple commutativity requirements for one finite diagram.
pub fn check_diagram_requirements(
    diagram_id: Id,
    requirements: &[DiagramCommutativityRequirement],
) -> DiagramCheckReport {
    let requirement_reports = requirements
        .iter()
        .map(|requirement| DiagramRequirementReport {
            requirement_id: requirement.id.clone(),
            report: check_diagram_commutativity(&requirement.left_path, &requirement.right_path),
        })
        .collect::<Vec<_>>();
    let commutes = requirement_reports
        .iter()
        .all(|requirement| requirement.report.commutes);

    DiagramCheckReport {
        diagram_id,
        commutes,
        requirement_reports,
    }
}

mod helpers;
use helpers::*;

#[cfg(test)]
mod tests;
