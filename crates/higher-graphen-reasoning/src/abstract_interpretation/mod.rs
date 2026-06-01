//! Conservative abstract interpretation summaries for HigherGraphen.

mod fixpoint;
pub use fixpoint::{
    run_fixpoint, AbstractDomain, AbstractEdge, AbstractElement, AbstractGraph, AbstractGraphNode,
    AbstractInterpretationReport, AbstractJoin, AbstractMembership, AbstractRegion,
    ConcreteWitnessSummary, FixpointObstruction, FixpointObstructionType, FixpointOptions,
    LossRegionKind, MembershipCheck, NodeAbstractState, NodeSeed, SoundnessStatus, WideningEvent,
    WitnessRelation,
};

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
