//! Conservative abstract interpretation summaries for HigherGraphen.

use higher_graphen_core::{CoreError, Id, Result, ReviewStatus};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;

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

fn required_text(field: &'static str, value: impl Into<String>) -> Result<String> {
    let raw = value.into();
    let normalized = raw.trim().to_owned();

    if normalized.is_empty() {
        return Err(CoreError::MalformedField {
            field: field.to_owned(),
            reason: "value must not be empty after trimming".to_owned(),
        });
    }

    Ok(normalized)
}

fn id_set(ids: impl IntoIterator<Item = Id>) -> BTreeSet<Id> {
    ids.into_iter().collect()
}

fn joined_vec<T>(left: &[T], right: &[T]) -> Vec<T>
where
    T: Clone + PartialEq,
{
    let mut items = left.to_vec();
    for item in right {
        if !items.contains(item) {
            items.push(item.clone());
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::{
        AbstractDomain, AbstractElement, AbstractMembership, AbstractRegion,
        ConcreteWitnessSummary, LossRegionKind, SoundnessStatus, WitnessRelation,
    };
    use higher_graphen_core::{Id, ReviewStatus};
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    fn assert_serde_contract<T>()
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
    }

    fn id(value: &str) -> Id {
        Id::new(value).expect("valid id")
    }

    #[test]
    fn join_intersects_definite_and_unions_possible_members() {
        let left = AbstractElement::exact(
            id("abstract/left"),
            AbstractDomain::DependencyReachability,
            [id("cell/a"), id("cell/b")],
        )
        .with_source_ids([id("analysis/left")]);
        let right = AbstractElement::exact(
            id("abstract/right"),
            AbstractDomain::DependencyReachability,
            [id("cell/b"), id("cell/c")],
        )
        .with_source_ids([id("analysis/right")]);

        let join = left
            .join(id("join/left-right"), id("abstract/joined"), &right)
            .expect("join same domain");

        assert_eq!(join.result.definite_concrete_ids, ids(["cell/b"]));
        assert_eq!(
            join.result.possible_concrete_ids,
            ids(["cell/a", "cell/b", "cell/c"])
        );
        assert_eq!(join.lost_definite_concrete_ids, ids(["cell/a", "cell/c"]));
        assert_eq!(
            join.possible_false_positive_concrete_ids,
            ids(["cell/a", "cell/c"])
        );
        assert_eq!(join.soundness, SoundnessStatus::Sound);
        assert_eq!(
            join.result.classify(&id("cell/a")),
            AbstractMembership::Possible
        );
        assert_eq!(
            join.result.classify(&id("cell/d")),
            AbstractMembership::Excluded
        );
        assert!(join.result.source_ids.contains(&id("abstract/left")));
        assert!(join.result.source_ids.contains(&id("abstract/right")));
    }

    #[test]
    fn unknown_soundness_never_turns_absence_into_a_proven_fact() {
        let element = AbstractElement::new(
            id("abstract/review"),
            AbstractDomain::ContextMembership,
            [id("context/a")],
            SoundnessStatus::Unknown,
        );

        assert_eq!(
            element.classify(&id("context/a")),
            AbstractMembership::Possible
        );
        assert_eq!(
            element.classify(&id("context/missing")),
            AbstractMembership::Unknown
        );
        assert!(element.is_conservative_record());
    }

    #[test]
    fn false_positive_unknown_regions_and_witnesses_are_explicit() {
        let false_positive = AbstractRegion::false_positive(
            id("region/false-positive"),
            "reachability summary may include cache-only edges",
            [id("cell/candidate")],
        )
        .expect("false-positive region")
        .with_source_ids([id("analysis/cache")])
        .with_review_status(ReviewStatus::Reviewed);
        let unknown = AbstractRegion::unknown(
            id("region/unknown"),
            "external dependency graph was summarized without expansion",
            [id("cell/external")],
        )
        .expect("unknown region");
        let witness = ConcreteWitnessSummary::new(
            id("witness/candidate"),
            WitnessRelation::FalsePositiveCandidate,
            "candidate appears only through a summarized cache edge",
        )
        .expect("witness")
        .with_concrete_ids([id("cell/candidate")])
        .with_source_ids([id("analysis/cache")]);

        let element = AbstractElement::new(
            id("abstract/large-space"),
            AbstractDomain::DependencyReachability,
            [id("cell/root"), id("cell/candidate")],
            SoundnessStatus::Sound,
        )
        .with_definite_concrete_ids([id("cell/root")])
        .expect("definite subset")
        .with_region(false_positive)
        .with_region(unknown)
        .with_concrete_witness(witness);

        assert_eq!(element.regions.len(), 2);
        assert_eq!(element.regions[0].kind, LossRegionKind::FalsePositive);
        assert_eq!(element.regions[0].review_status, ReviewStatus::Reviewed);
        assert_eq!(
            element.classify(&id("cell/external")),
            AbstractMembership::UnknownRegion
        );
        assert_eq!(
            element.concrete_witnesses[0].relation,
            WitnessRelation::FalsePositiveCandidate
        );
    }

    #[test]
    fn serde_rejects_malformed_abstract_elements() {
        let definite_not_possible = json!({
            "id": "abstract/bad",
            "domain": "dependency_reachability",
            "definite_concrete_ids": ["cell/a"],
            "possible_concrete_ids": ["cell/b"],
            "soundness": "sound"
        });
        let unknown_field = json!({
            "id": "abstract/bad",
            "domain": "dependency_reachability",
            "possible_concrete_ids": ["cell/a"],
            "soundness": "sound",
            "unexpected": true
        });

        assert!(
            serde_json::from_value::<AbstractElement>(definite_not_possible).is_err(),
            "definite ids outside possible ids must be rejected"
        );
        assert!(
            serde_json::from_value::<AbstractElement>(unknown_field).is_err(),
            "unknown fields must be rejected"
        );
    }

    #[test]
    fn constructors_validate_text_and_join_domains() {
        assert!(AbstractRegion::unknown(id("region/bad"), " ", []).is_err());
        assert!(
            ConcreteWitnessSummary::new(id("witness/bad"), WitnessRelation::UnknownRegion, "")
                .is_err()
        );

        let reachability = AbstractElement::new(
            id("abstract/reachability"),
            AbstractDomain::DependencyReachability,
            [id("cell/a")],
            SoundnessStatus::Sound,
        );
        let context = AbstractElement::new(
            id("abstract/context"),
            AbstractDomain::ContextMembership,
            [id("context/a")],
            SoundnessStatus::Sound,
        );

        assert!(
            reachability
                .join(id("join/mismatch"), id("abstract/mismatch"), &context)
                .is_err(),
            "different domains must not be joined"
        );
    }

    #[test]
    fn public_types_implement_serde_contracts() {
        assert_serde_contract::<AbstractDomain>();
        assert_serde_contract::<SoundnessStatus>();
        assert_serde_contract::<LossRegionKind>();
        assert_serde_contract::<AbstractMembership>();
        assert_serde_contract::<AbstractRegion>();
        assert_serde_contract::<WitnessRelation>();
        assert_serde_contract::<ConcreteWitnessSummary>();
        assert_serde_contract::<AbstractElement>();
        assert_serde_contract::<super::AbstractJoin>();
    }

    fn ids<const N: usize>(values: [&str; N]) -> std::collections::BTreeSet<Id> {
        values.into_iter().map(id).collect()
    }
}
