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
    pub cell_mapping: CellMapping,
    /// Explicit source-relation to target-relation mappings.
    pub relation_mapping: RelationMapping,
    /// Invariants known to be preserved by this morphism.
    pub preserved_invariant_ids: Vec<Id>,
    /// Source elements known to be lost by this morphism.
    pub lost_structure: Vec<LostStructure>,
    /// Distortions known to be introduced by this morphism.
    pub distortion: Vec<Distortion>,
    /// Morphism identifiers declared compatible by metadata.
    pub composable_with: Vec<Id>,
    /// Source and review metadata for this morphism.
    pub provenance: Provenance,
}

/// Deterministic preservation check result for selected invariant IDs.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreservationReport {
    /// Selected invariant IDs found in the morphism preserved set.
    pub preserved: Vec<Id>,
    /// Selected invariant IDs absent from the morphism preserved set.
    pub violated: Vec<Id>,
    /// Lost structure recorded on the checked morphism.
    pub lost_structure: Vec<LostStructure>,
    /// Distortion recorded on the checked morphism.
    pub distortion: Vec<Distortion>,
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

    CompositionResult::Composed {
        morphism: Box::new(Morphism {
            id: composed_id,
            source_space_id: first.source_space_id.clone(),
            target_space_id: second.target_space_id.clone(),
            name: name.into(),
            morphism_type,
            cell_mapping: compose_mapping(&first.cell_mapping, &second.cell_mapping),
            relation_mapping: compose_mapping(&first.relation_mapping, &second.relation_mapping),
            preserved_invariant_ids: intersect_ids(
                &first.preserved_invariant_ids,
                &second.preserved_invariant_ids,
            ),
            lost_structure: concat_records(&first.lost_structure, &second.lost_structure),
            distortion: concat_records(&first.distortion, &second.distortion),
            composable_with: Vec::new(),
            provenance,
        }),
    }
}

fn compose_mapping(first: &BTreeMap<Id, Id>, second: &BTreeMap<Id, Id>) -> BTreeMap<Id, Id> {
    first
        .iter()
        .filter_map(|(source_id, intermediate_id)| {
            second
                .get(intermediate_id)
                .map(|target_id| (source_id.clone(), target_id.clone()))
        })
        .collect()
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
mod tests {
    use super::*;
    use higher_graphen_core::{Confidence, ReviewStatus, SourceKind, SourceRef};

    #[test]
    fn composition_succeeds_for_compatible_spaces() {
        let first = fixture_morphism(
            "first",
            "space/a",
            "space/b",
            [("cell/a1", "cell/b1")],
            [("rel/a1", "rel/b1")],
            ["invariant/a", "invariant/shared"],
        );
        let second = fixture_morphism(
            "second",
            "space/b",
            "space/c",
            [("cell/b1", "cell/c1")],
            [("rel/b1", "rel/c1")],
            ["invariant/shared", "invariant/c"],
        );

        let result = compose_morphisms(
            &first,
            &second,
            id("first-then-second"),
            "first then second",
            MorphismType::Translation,
            provenance(),
        );

        let CompositionResult::Composed { morphism } = result else {
            panic!("expected compatible morphisms to compose");
        };

        assert_eq!(morphism.source_space_id, id("space/a"));
        assert_eq!(morphism.target_space_id, id("space/c"));
        assert_eq!(morphism.cell_mapping[&id("cell/a1")], id("cell/c1"));
        assert_eq!(morphism.relation_mapping[&id("rel/a1")], id("rel/c1"));
        assert_eq!(
            morphism.preserved_invariant_ids,
            vec![id("invariant/shared")]
        );
        assert_eq!(morphism.lost_structure.len(), 2);
        assert_eq!(morphism.distortion.len(), 2);
    }

    #[test]
    fn composition_rejects_incompatible_spaces() {
        let first = fixture_morphism(
            "first",
            "space/a",
            "space/b",
            [("cell/a1", "cell/b1")],
            [("rel/a1", "rel/b1")],
            ["invariant/a"],
        );
        let second = fixture_morphism(
            "second",
            "space/x",
            "space/c",
            [("cell/x1", "cell/c1")],
            [("rel/x1", "rel/c1")],
            ["invariant/x"],
        );

        let result = first.compose_with(
            &second,
            id("invalid"),
            "invalid composition",
            MorphismType::Translation,
            provenance(),
        );

        assert_eq!(
            result,
            CompositionResult::IncompatibleSpace {
                first_morphism_id: id("first"),
                second_morphism_id: id("second"),
                first_target_space_id: id("space/b"),
                second_source_space_id: id("space/x"),
            }
        );
    }

    #[test]
    fn composition_does_not_infer_unmatched_intermediate_mappings() {
        let first = fixture_morphism(
            "first",
            "space/a",
            "space/b",
            [("cell/a1", "cell/b1"), ("cell/a2", "cell/b2")],
            [("rel/a1", "rel/b1"), ("rel/a2", "rel/b2")],
            ["invariant/shared"],
        );
        let second = fixture_morphism(
            "second",
            "space/b",
            "space/c",
            [("cell/b1", "cell/c1")],
            [("rel/b1", "rel/c1")],
            ["invariant/shared"],
        );

        let CompositionResult::Composed { morphism } = compose_morphisms(
            &first,
            &second,
            id("composed"),
            "composed",
            MorphismType::Projection,
            provenance(),
        ) else {
            panic!("expected compatible morphisms to compose");
        };

        assert_eq!(morphism.cell_mapping.len(), 1);
        assert_eq!(morphism.relation_mapping.len(), 1);
        assert!(!morphism.cell_mapping.contains_key(&id("cell/a2")));
        assert!(!morphism.relation_mapping.contains_key(&id("rel/a2")));
    }

    #[test]
    fn preservation_check_sorts_and_deduplicates_selected_invariants() {
        let morphism = fixture_morphism(
            "morphism",
            "space/a",
            "space/b",
            [("cell/a1", "cell/b1")],
            [("rel/a1", "rel/b1")],
            ["invariant/b", "invariant/a"],
        );

        let report = morphism.check_preservation([
            id("invariant/c"),
            id("invariant/a"),
            id("invariant/a"),
            id("invariant/b"),
        ]);

        assert_eq!(report.preserved, vec![id("invariant/a"), id("invariant/b")]);
        assert_eq!(report.violated, vec![id("invariant/c")]);
        assert_eq!(report.lost_structure, morphism.lost_structure);
        assert_eq!(report.distortion, morphism.distortion);
    }

    fn fixture_morphism<const C: usize, const R: usize, const I: usize>(
        morphism_id: &str,
        source_space_id: &str,
        target_space_id: &str,
        cell_pairs: [(&str, &str); C],
        relation_pairs: [(&str, &str); R],
        invariant_ids: [&str; I],
    ) -> Morphism {
        Morphism {
            id: id(morphism_id),
            source_space_id: id(source_space_id),
            target_space_id: id(target_space_id),
            name: morphism_id.to_owned(),
            morphism_type: MorphismType::Translation,
            cell_mapping: mapping(cell_pairs),
            relation_mapping: mapping(relation_pairs),
            preserved_invariant_ids: invariant_ids.into_iter().map(id).collect(),
            lost_structure: vec![LostStructure {
                source_element_id: id(format!("{morphism_id}/lost")),
                reason: "fixture loss".to_owned(),
                severity: Severity::Low,
            }],
            distortion: vec![Distortion {
                source_element_id: id(format!("{morphism_id}/source")),
                target_element_id: id(format!("{morphism_id}/target")),
                description: "fixture distortion".to_owned(),
                severity: Severity::Medium,
            }],
            composable_with: Vec::new(),
            provenance: provenance(),
        }
    }

    fn mapping<const N: usize>(pairs: [(&str, &str); N]) -> BTreeMap<Id, Id> {
        pairs
            .into_iter()
            .map(|(source_id, target_id)| (id(source_id), id(target_id)))
            .collect()
    }

    fn provenance() -> Provenance {
        Provenance::new(
            SourceRef::new(SourceKind::custom("morphism-test").expect("valid custom source kind")),
            Confidence::new(1.0).expect("valid confidence"),
        )
        .with_review_status(ReviewStatus::Accepted)
    }

    fn id(value: impl AsRef<str>) -> Id {
        Id::new(value.as_ref()).expect("valid id")
    }
}
