//! Shared abstract-interpretation data types and fixpoint inputs.

use super::helpers::{id_set, joined_vec, malformed, required_text};
use higher_graphen_core::{CoreError, Id, Result, ReviewStatus};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Stable domain category for an abstract element.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AbstractDomain {
    /// Possible dependency reachability across a large graph.
    DependencyReachability,
    /// Possible membership in one or more contexts.
    ContextMembership,
    /// Possible membership in a large region or search space.
    RegionMembership,
    /// Downstream-owned domain name.
    Custom(String),
}

/// Soundness status for an over-approximation record.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundnessStatus {
    /// The record may be used as a no-false-negative over-approximation.
    Sound,
    /// The record remains conservative for review, but absence is not proof.
    Unknown,
    /// A known gap invalidates conservative use.
    Unsound,
}

impl SoundnessStatus {
    /// Combines two soundness states for a join.
    #[must_use]
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unsound, _) | (_, Self::Unsound) => Self::Unsound,
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::Sound, Self::Sound) => Self::Sound,
        }
    }

    /// Returns true when absence from `possible_concrete_ids` can be treated as excluded.
    #[must_use]
    pub fn permits_absence_proofs(self) -> bool {
        matches!(self, Self::Sound)
    }

    /// Returns true when this status does not knowingly break no-false-negative use.
    #[must_use]
    pub fn is_not_known_unsound(self) -> bool {
        !matches!(self, Self::Unsound)
    }
}

/// Precision-loss category carried by a region record.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LossRegionKind {
    /// The region may contain candidates that are not concrete members.
    FalsePositive,
    /// The region was not resolved precisely enough for absence or presence proof.
    Unknown,
}

/// How an element classifies a concrete identifier.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AbstractMembership {
    /// Present in every represented concrete state.
    Definite,
    /// Present in at least one represented concrete state, or kept to avoid false negatives.
    Possible,
    /// Covered by an explicit unknown precision-loss region.
    UnknownRegion,
    /// Not listed and the abstraction is sound, so absence can be used.
    Excluded,
    /// The abstraction is globally unknown, so absence is not proof.
    Unknown,
    /// The abstraction is known unsound and cannot classify conservatively.
    Unsound,
}

/// Region where the abstract element deliberately preserves imprecision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbstractRegion {
    /// Region identifier.
    pub id: Id,
    /// Precision-loss category.
    pub kind: LossRegionKind,
    /// Human-readable explanation of the region.
    pub summary: String,
    /// Concrete identifiers affected by this imprecision record.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub affected_concrete_ids: BTreeSet<Id>,
    /// Source records or analyses that produced this region.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub source_ids: BTreeSet<Id>,
    /// Human or workflow review status for this loss record.
    #[serde(default)]
    pub review_status: ReviewStatus,
}

impl AbstractRegion {
    /// Creates a false-positive region with validated text.
    pub fn false_positive(
        id: Id,
        summary: impl Into<String>,
        affected_concrete_ids: impl IntoIterator<Item = Id>,
    ) -> Result<Self> {
        Self::new(
            id,
            LossRegionKind::FalsePositive,
            summary,
            affected_concrete_ids,
        )
    }

    /// Creates an unknown region with validated text.
    pub fn unknown(
        id: Id,
        summary: impl Into<String>,
        affected_concrete_ids: impl IntoIterator<Item = Id>,
    ) -> Result<Self> {
        Self::new(id, LossRegionKind::Unknown, summary, affected_concrete_ids)
    }

    fn new(
        id: Id,
        kind: LossRegionKind,
        summary: impl Into<String>,
        affected_concrete_ids: impl IntoIterator<Item = Id>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            kind,
            summary: required_text("region.summary", summary)?,
            affected_concrete_ids: id_set(affected_concrete_ids),
            source_ids: BTreeSet::new(),
            review_status: ReviewStatus::Unreviewed,
        })
    }

    /// Returns this region with source records attached.
    #[must_use]
    pub fn with_source_ids(mut self, source_ids: impl IntoIterator<Item = Id>) -> Self {
        self.source_ids = id_set(source_ids);
        self
    }

    /// Returns this region with an explicit review status.
    #[must_use]
    pub fn with_review_status(mut self, review_status: ReviewStatus) -> Self {
        self.review_status = review_status;
        self
    }

    fn covers(&self, concrete_id: &Id) -> bool {
        self.affected_concrete_ids.contains(concrete_id)
    }
}

/// Relationship between a concrete witness and the abstraction.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessRelation {
    /// Witness supports a definite concrete member.
    DefiniteConcrete,
    /// Witness supports a possible member retained by the over-approximation.
    PossibleConcrete,
    /// Witness explains why a possible member may be a false positive.
    FalsePositiveCandidate,
    /// Witness explains an unknown region.
    UnknownRegion,
}

/// Compact evidence that lets consumers inspect concrete examples without expanding a large space.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConcreteWitnessSummary {
    /// Witness identifier.
    pub id: Id,
    /// How the witness relates to the abstraction.
    pub relation: WitnessRelation,
    /// Human-readable witness summary.
    pub summary: String,
    /// Concrete identifiers represented by the witness.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub concrete_ids: BTreeSet<Id>,
    /// Source records or analyses that produced this witness.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub source_ids: BTreeSet<Id>,
    /// Human or workflow review status for this witness.
    #[serde(default)]
    pub review_status: ReviewStatus,
}

impl ConcreteWitnessSummary {
    /// Creates a concrete witness summary with validated text.
    pub fn new(id: Id, relation: WitnessRelation, summary: impl Into<String>) -> Result<Self> {
        Ok(Self {
            id,
            relation,
            summary: required_text("witness.summary", summary)?,
            concrete_ids: BTreeSet::new(),
            source_ids: BTreeSet::new(),
            review_status: ReviewStatus::Unreviewed,
        })
    }

    /// Returns this witness with represented concrete identifiers.
    #[must_use]
    pub fn with_concrete_ids(mut self, concrete_ids: impl IntoIterator<Item = Id>) -> Self {
        self.concrete_ids = id_set(concrete_ids);
        self
    }

    /// Returns this witness with source records attached.
    #[must_use]
    pub fn with_source_ids(mut self, source_ids: impl IntoIterator<Item = Id>) -> Self {
        self.source_ids = id_set(source_ids);
        self
    }

    /// Returns this witness with an explicit review status.
    #[must_use]
    pub fn with_review_status(mut self, review_status: ReviewStatus) -> Self {
        self.review_status = review_status;
        self
    }
}

/// Abstract element representing a conservative summary of many concrete identifiers.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbstractElement {
    /// Element identifier.
    pub id: Id,
    /// Domain in which this abstract element is meaningful.
    pub domain: AbstractDomain,
    /// Source records or analyses used to create the element.
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub source_ids: BTreeSet<Id>,
    /// Concrete identifiers known to be present in every represented concrete state.
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub definite_concrete_ids: BTreeSet<Id>,
    /// Concrete identifiers retained to avoid false negatives.
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub possible_concrete_ids: BTreeSet<Id>,
    /// Soundness status for this over-approximation.
    pub soundness: SoundnessStatus,
    /// Explicit false-positive or unknown regions.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<AbstractRegion>,
    /// Compact concrete examples backing the abstraction.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub concrete_witnesses: Vec<ConcreteWitnessSummary>,
}

impl AbstractElement {
    /// Creates an abstract element with possible concrete identifiers and no definite facts.
    pub fn new(
        id: Id,
        domain: AbstractDomain,
        possible_concrete_ids: impl IntoIterator<Item = Id>,
        soundness: SoundnessStatus,
    ) -> Self {
        Self {
            id,
            domain,
            source_ids: BTreeSet::new(),
            definite_concrete_ids: BTreeSet::new(),
            possible_concrete_ids: id_set(possible_concrete_ids),
            soundness,
            regions: Vec::new(),
            concrete_witnesses: Vec::new(),
        }
    }

    /// Creates a sound exact element where every possible identifier is definite.
    pub fn exact(
        id: Id,
        domain: AbstractDomain,
        concrete_ids: impl IntoIterator<Item = Id>,
    ) -> Self {
        let concrete_ids = id_set(concrete_ids);

        Self {
            id,
            domain,
            source_ids: BTreeSet::new(),
            definite_concrete_ids: concrete_ids.clone(),
            possible_concrete_ids: concrete_ids,
            soundness: SoundnessStatus::Sound,
            regions: Vec::new(),
            concrete_witnesses: Vec::new(),
        }
    }

    /// Replaces the source identifiers.
    #[must_use]
    pub fn with_source_ids(mut self, source_ids: impl IntoIterator<Item = Id>) -> Self {
        self.source_ids = id_set(source_ids);
        self
    }

    /// Replaces the definite concrete identifiers after checking subset consistency.
    pub fn with_definite_concrete_ids(
        mut self,
        definite_concrete_ids: impl IntoIterator<Item = Id>,
    ) -> Result<Self> {
        self.definite_concrete_ids = id_set(definite_concrete_ids);
        self.validate()?;
        Ok(self)
    }

    /// Adds a precision-loss region.
    #[must_use]
    pub fn with_region(mut self, region: AbstractRegion) -> Self {
        if !self.regions.contains(&region) {
            self.regions.push(region);
        }
        self
    }

    /// Adds a concrete witness summary.
    #[must_use]
    pub fn with_concrete_witness(mut self, witness: ConcreteWitnessSummary) -> Self {
        if !self.concrete_witnesses.contains(&witness) {
            self.concrete_witnesses.push(witness);
        }
        self
    }

    /// Checks structural invariants that make the record conservatively interpretable.
    pub fn validate(&self) -> Result<()> {
        if !self
            .definite_concrete_ids
            .is_subset(&self.possible_concrete_ids)
        {
            return Err(CoreError::MalformedField {
                field: "definite_concrete_ids".to_owned(),
                reason: "definite concrete ids must be a subset of possible concrete ids"
                    .to_owned(),
            });
        }

        Ok(())
    }

    /// Returns true when this element has no known soundness failure and passes invariants.
    #[must_use]
    pub fn is_conservative_record(&self) -> bool {
        self.soundness.is_not_known_unsound() && self.validate().is_ok()
    }

    /// Classifies a concrete identifier under this abstract element.
    #[must_use]
    pub fn classify(&self, concrete_id: &Id) -> AbstractMembership {
        if matches!(self.soundness, SoundnessStatus::Unsound) {
            return AbstractMembership::Unsound;
        }

        if self.definite_concrete_ids.contains(concrete_id) {
            return AbstractMembership::Definite;
        }

        if self.possible_concrete_ids.contains(concrete_id) {
            return AbstractMembership::Possible;
        }

        if self.regions.iter().any(|region| {
            matches!(region.kind, LossRegionKind::Unknown) && region.covers(concrete_id)
        }) {
            return AbstractMembership::UnknownRegion;
        }

        if self.soundness.permits_absence_proofs() {
            AbstractMembership::Excluded
        } else {
            AbstractMembership::Unknown
        }
    }

    /// Joins this element with another element from the same domain.
    pub fn join(&self, join_id: Id, result_id: Id, other: &Self) -> Result<AbstractJoin> {
        AbstractJoin::new(join_id, result_id, self, other)
    }
}

impl<'de> Deserialize<'de> for AbstractElement {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            id: Id,
            domain: AbstractDomain,
            #[serde(default)]
            source_ids: BTreeSet<Id>,
            #[serde(default)]
            definite_concrete_ids: BTreeSet<Id>,
            #[serde(default)]
            possible_concrete_ids: BTreeSet<Id>,
            soundness: SoundnessStatus,
            #[serde(default)]
            regions: Vec<AbstractRegion>,
            #[serde(default)]
            concrete_witnesses: Vec<ConcreteWitnessSummary>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let element = Self {
            id: wire.id,
            domain: wire.domain,
            source_ids: wire.source_ids,
            definite_concrete_ids: wire.definite_concrete_ids,
            possible_concrete_ids: wire.possible_concrete_ids,
            soundness: wire.soundness,
            regions: wire.regions,
            concrete_witnesses: wire.concrete_witnesses,
        };
        element.validate().map_err(serde::de::Error::custom)?;
        Ok(element)
    }
}

/// Deterministic record of a join between two abstract elements.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbstractJoin {
    /// Join identifier.
    pub id: Id,
    /// Left input element.
    pub left_element_id: Id,
    /// Right input element.
    pub right_element_id: Id,
    /// Joined result element.
    pub result: AbstractElement,
    /// Definite facts demoted by the join because they were not common to both sides.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub lost_definite_concrete_ids: BTreeSet<Id>,
    /// Possible members not proven definite after the join.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub possible_false_positive_concrete_ids: BTreeSet<Id>,
    /// Combined soundness status.
    pub soundness: SoundnessStatus,
}

impl AbstractJoin {
    /// Creates a deterministic join record.
    pub fn new(
        id: Id,
        result_id: Id,
        left: &AbstractElement,
        right: &AbstractElement,
    ) -> Result<Self> {
        if left.domain != right.domain {
            return Err(CoreError::MalformedField {
                field: "join.domain".to_owned(),
                reason: "cannot join abstract elements from different domains".to_owned(),
            });
        }

        left.validate()?;
        right.validate()?;

        let definite_concrete_ids = left
            .definite_concrete_ids
            .intersection(&right.definite_concrete_ids)
            .cloned()
            .collect::<BTreeSet<_>>();
        let possible_concrete_ids = left
            .possible_concrete_ids
            .union(&right.possible_concrete_ids)
            .cloned()
            .collect::<BTreeSet<_>>();
        let previously_definite = left
            .definite_concrete_ids
            .union(&right.definite_concrete_ids)
            .cloned()
            .collect::<BTreeSet<_>>();
        let lost_definite_concrete_ids = previously_definite
            .difference(&definite_concrete_ids)
            .cloned()
            .collect::<BTreeSet<_>>();
        let possible_false_positive_concrete_ids = possible_concrete_ids
            .difference(&definite_concrete_ids)
            .cloned()
            .collect::<BTreeSet<_>>();
        let soundness = left.soundness.join(right.soundness);

        let mut result = AbstractElement {
            id: result_id,
            domain: left.domain.clone(),
            source_ids: left.source_ids.union(&right.source_ids).cloned().collect(),
            definite_concrete_ids,
            possible_concrete_ids,
            soundness,
            regions: joined_vec(&left.regions, &right.regions),
            concrete_witnesses: joined_vec(&left.concrete_witnesses, &right.concrete_witnesses),
        };
        result.source_ids.insert(left.id.clone());
        result.source_ids.insert(right.id.clone());
        result.validate()?;

        Ok(Self {
            id,
            left_element_id: left.id.clone(),
            right_element_id: right.id.clone(),
            result,
            lost_definite_concrete_ids,
            possible_false_positive_concrete_ids,
            soundness,
        })
    }
}

/// One directed edge with a monotone `gen`-only transfer.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbstractEdge {
    /// Successor node reached by this edge.
    pub target_node_id: Id,
    /// Concrete identifiers introduced into both the must-set and may-set.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub gen_definite: BTreeSet<Id>,
    /// Concrete identifiers introduced into the may-set only.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub gen_possible: BTreeSet<Id>,
}

impl AbstractEdge {
    /// Creates an identity-transfer edge (no introduced identifiers).
    #[must_use]
    pub fn new(target_node_id: Id) -> Self {
        Self {
            target_node_id,
            gen_definite: BTreeSet::new(),
            gen_possible: BTreeSet::new(),
        }
    }

    /// Returns this edge with concrete identifiers added to the must-set and may-set.
    #[must_use]
    pub fn with_gen_definite(mut self, gen_definite: impl IntoIterator<Item = Id>) -> Self {
        self.gen_definite = gen_definite.into_iter().collect();
        self
    }

    /// Returns this edge with concrete identifiers added to the may-set only.
    #[must_use]
    pub fn with_gen_possible(mut self, gen_possible: impl IntoIterator<Item = Id>) -> Self {
        self.gen_possible = gen_possible.into_iter().collect();
        self
    }

    /// Applies the monotone transfer `f_edge` to an incoming state.
    ///
    /// `gen_definite` is unioned into both sets; `gen_possible` is unioned into
    /// the may-set only. The result keeps the must-set a subset of the may-set,
    /// so it is a valid [`AbstractElement`].
    pub(super) fn transfer(&self, result_id: Id, incoming: &AbstractElement) -> AbstractElement {
        let mut definite = incoming.definite_concrete_ids.clone();
        definite.extend(self.gen_definite.iter().cloned());
        let mut possible = incoming.possible_concrete_ids.clone();
        possible.extend(self.gen_definite.iter().cloned());
        possible.extend(self.gen_possible.iter().cloned());

        let mut state = AbstractElement::new(
            result_id,
            incoming.domain.clone(),
            possible,
            incoming.soundness,
        );
        state.definite_concrete_ids = definite;
        state.source_ids = incoming.source_ids.clone();
        state
    }
}

/// One node of the explicit finite directed graph.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbstractGraphNode {
    /// Stable node identifier.
    pub node_id: Id,
    /// Outgoing edges. Deduplicated and sorted on graph construction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub successors: Vec<AbstractEdge>,
}

impl AbstractGraphNode {
    /// Creates a node with the supplied outgoing edges.
    #[must_use]
    pub fn new(node_id: Id, successors: impl IntoIterator<Item = AbstractEdge>) -> Self {
        Self {
            node_id,
            successors: successors.into_iter().collect(),
        }
    }
}

/// Seed abstract state attached to a node before iteration starts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeSeed {
    /// Node the seed applies to.
    pub node_id: Id,
    /// Initial may-set (possible concrete identifiers).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub possible_concrete_ids: BTreeSet<Id>,
    /// Initial must-set (definite concrete identifiers); must be a subset of the may-set.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub definite_concrete_ids: BTreeSet<Id>,
}

impl NodeSeed {
    /// Creates a seed with only possible (may-set) identifiers.
    #[must_use]
    pub fn possible(node_id: Id, possible_concrete_ids: impl IntoIterator<Item = Id>) -> Self {
        Self {
            node_id,
            possible_concrete_ids: possible_concrete_ids.into_iter().collect(),
            definite_concrete_ids: BTreeSet::new(),
        }
    }

    /// Creates an exact seed where every possible identifier is also definite.
    #[must_use]
    pub fn exact(node_id: Id, concrete_ids: impl IntoIterator<Item = Id>) -> Self {
        let concrete_ids: BTreeSet<Id> = concrete_ids.into_iter().collect();
        Self {
            node_id,
            possible_concrete_ids: concrete_ids.clone(),
            definite_concrete_ids: concrete_ids,
        }
    }

    /// Returns this seed with definite (must-set) identifiers attached.
    #[must_use]
    pub fn with_definite_concrete_ids(
        mut self,
        definite_concrete_ids: impl IntoIterator<Item = Id>,
    ) -> Self {
        self.definite_concrete_ids = definite_concrete_ids.into_iter().collect();
        self
    }
}

/// Explicit finite directed graph plus seed states for the fixpoint solver.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbstractGraph {
    /// Single domain shared by every node state (joins require equal domains).
    pub domain: AbstractDomain,
    /// Nodes with their outgoing adjacency.
    pub nodes: Vec<AbstractGraphNode>,
    /// Seed abstract states.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seeds: Vec<NodeSeed>,
    /// Soundness assumption joined into every node state.
    pub soundness: SoundnessStatus,
}

impl AbstractGraph {
    /// Creates a graph in the supplied domain.
    #[must_use]
    pub fn new(domain: AbstractDomain, soundness: SoundnessStatus) -> Self {
        Self {
            domain,
            nodes: Vec::new(),
            seeds: Vec::new(),
            soundness,
        }
    }

    /// Returns this graph with the supplied nodes appended.
    #[must_use]
    pub fn with_nodes(mut self, nodes: impl IntoIterator<Item = AbstractGraphNode>) -> Self {
        self.nodes.extend(nodes);
        self
    }

    /// Returns this graph with the supplied seeds appended.
    #[must_use]
    pub fn with_seeds(mut self, seeds: impl IntoIterator<Item = NodeSeed>) -> Self {
        self.seeds.extend(seeds);
        self
    }

    /// Normalizes nodes and validates that edges and seeds reference known nodes.
    ///
    /// Produces the deterministic adjacency map and the seed states. Returns a
    /// `MalformedField` error when an edge target or a seed node is unknown, or
    /// when a seed's must-set is not a subset of its may-set.
    pub(super) fn normalize(&self) -> Result<NormalizedGraph> {
        let mut adjacency: BTreeMap<Id, BTreeSet<AbstractEdge>> = BTreeMap::new();
        for node in &self.nodes {
            adjacency
                .entry(node.node_id.clone())
                .or_default()
                .extend(node.successors.iter().cloned());
        }

        let node_ids: BTreeSet<Id> = adjacency.keys().cloned().collect();
        for edges in adjacency.values() {
            for edge in edges {
                if !node_ids.contains(&edge.target_node_id) {
                    return Err(malformed(
                        "edge.target_node_id",
                        format!(
                            "edge target {} is not a declared node",
                            edge.target_node_id.as_str()
                        ),
                    ));
                }
            }
        }

        let mut seed_states: BTreeMap<Id, AbstractElement> = BTreeMap::new();
        for seed in &self.seeds {
            if !node_ids.contains(&seed.node_id) {
                return Err(malformed(
                    "seed.node_id",
                    format!("seed node {} is not a declared node", seed.node_id.as_str()),
                ));
            }
            let mut state = AbstractElement::new(
                seed.node_id.clone(),
                self.domain.clone(),
                seed.possible_concrete_ids.iter().cloned(),
                self.soundness,
            );
            state.definite_concrete_ids = seed.definite_concrete_ids.clone();
            state.validate()?;
            seed_states.insert(seed.node_id.clone(), state);
        }

        Ok(NormalizedGraph {
            adjacency,
            seed_states,
        })
    }
}

/// Normalized, validated adjacency and seed states.
pub(super) struct NormalizedGraph {
    pub(super) adjacency: BTreeMap<Id, BTreeSet<AbstractEdge>>,
    pub(super) seed_states: BTreeMap<Id, AbstractElement>,
}

/// Worklist and widening controls.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixpointOptions {
    /// A node is widened only after it is popped strictly more than this many times.
    pub widen_threshold: usize,
    /// May-set a widened node is relaxed to. Empty means widening only shrinks the must-set.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub widen_top: BTreeSet<Id>,
    /// Hard bound on total worklist pops before reporting `iteration_limit_exceeded`.
    pub max_iterations: usize,
}

impl Default for FixpointOptions {
    fn default() -> Self {
        Self {
            widen_threshold: 3,
            widen_top: BTreeSet::new(),
            max_iterations: 10_000,
        }
    }
}

impl FixpointOptions {
    /// Creates default options (widen after 3 revisits, no universe, 10000-pop bound).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns these options with a revisit threshold.
    #[must_use]
    pub fn with_widen_threshold(mut self, widen_threshold: usize) -> Self {
        self.widen_threshold = widen_threshold;
        self
    }

    /// Returns these options with an explicit widening universe (top may-set).
    #[must_use]
    pub fn with_widen_top(mut self, widen_top: impl IntoIterator<Item = Id>) -> Self {
        self.widen_top = widen_top.into_iter().collect();
        self
    }

    /// Returns these options with an iteration bound.
    #[must_use]
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }
}

/// A selected per-node membership check evaluated against the final state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipCheck {
    /// Stable check identifier.
    pub id: Id,
    /// Node whose final abstract state is queried.
    pub node_id: Id,
    /// Concrete identifier whose membership is checked.
    pub concrete_id: Id,
}

impl MembershipCheck {
    /// Creates a membership check.
    #[must_use]
    pub fn new(id: Id, node_id: Id, concrete_id: Id) -> Self {
        Self {
            id,
            node_id,
            concrete_id,
        }
    }
}

/// Stable obstruction category emitted by the fixpoint engine.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FixpointObstructionType {
    /// Two states to merge are in different domains, so the join is undefined.
    MissingJoinOperation,
    /// A seed or derived state is known unsound; conservative absence reasoning is invalid.
    UnsoundAbstraction,
    /// A widening step moved a required distinction into imprecision.
    WideningLostRequiredDistinction,
    /// A selected check is unknown and requires a concrete witness before use.
    UnknownRegionRequiresWitness,
    /// The iteration bound was hit before a fixed point was reached.
    IterationLimitExceeded,
}

/// Structured fixpoint obstruction.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixpointObstruction {
    /// Obstruction category.
    pub obstruction_type: FixpointObstructionType,
    /// Human-readable diagnostic.
    pub reason: String,
    /// Node the obstruction is attached to, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<Id>,
}

/// Record of one widening application at a node.
///
/// Widening relaxes only the may-set toward the configured universe; the
/// must-set is monotone decreasing under join and converges without widening,
/// so it is never widened. The added may-set ids are recorded so a reviewer can
/// see exactly which exclusions the widening sacrificed.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WideningEvent {
    /// Node that was widened.
    pub node_id: Id,
    /// Worklist iteration (total pop count) at which widening was applied.
    pub iteration: usize,
    /// May-set identifiers added by relaxing toward the widening universe.
    pub widened_possible_ids: BTreeSet<Id>,
}

/// Per-node final abstract state.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeAbstractState {
    /// Node the state belongs to.
    pub node_id: Id,
    /// Final abstract state at the node. Its element id equals the node id.
    pub state: AbstractElement,
}

/// Structured abstract-interpretation report (see math-extension-kernels.md).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AbstractInterpretationReport {
    /// Analysis identifier.
    pub analysis_id: Id,
    /// Per-node final abstract states, sorted by node id.
    pub node_states: Vec<NodeAbstractState>,
    /// True when the solver reached a fixed point within the iteration bound.
    pub reached_fixpoint: bool,
    /// Total worklist pops performed.
    pub iterations: usize,
    /// Checks whose concrete id is definitely a member at the node, sorted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub definitely_satisfied_check_ids: Vec<Id>,
    /// Checks whose concrete id is only possibly a member (the safe default), sorted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub possibly_violated_check_ids: Vec<Id>,
    /// Nodes whose final state is unsound or carries an unknown region, sorted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknown_region_node_ids: Vec<Id>,
    /// Widening applications, sorted by (node id, iteration).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub widening_events: Vec<WideningEvent>,
    /// Obstructions, sorted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<FixpointObstruction>,
    /// Explicit information-loss declarations, sorted and deduplicated.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub information_loss: Vec<String>,
}

impl AbstractInterpretationReport {
    /// Returns the final abstract state at a node, when present.
    #[must_use]
    pub fn state(&self, node_id: &Id) -> Option<&AbstractElement> {
        self.node_states
            .iter()
            .find(|entry| &entry.node_id == node_id)
            .map(|entry| &entry.state)
    }

    /// Returns true when the solver reached a fixed point and no obstruction was raised.
    #[must_use]
    pub fn is_conclusive(&self) -> bool {
        self.reached_fixpoint && self.obstructions.is_empty()
    }
}
