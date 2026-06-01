use super::helpers::*;
use super::*;

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
