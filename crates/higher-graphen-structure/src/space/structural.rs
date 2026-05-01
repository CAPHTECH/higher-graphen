//! Generic finite structural-observation analysis.

use higher_graphen_core::Id;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Generic structural role assigned to a finite observation.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuralRole {
    /// Observation about a subject's boundary.
    Boundary,
    /// Observation about a subject's coboundary.
    Coboundary,
    /// Observation about a subject's incidence relation.
    Incidence,
    /// Observation about a subject's composition.
    Composition,
    /// Observation about a subject's projection.
    Projection,
    /// Observation about a subject's contract.
    Contract,
    /// Observation that supports interpretation of other observations.
    Evidence,
}

impl StructuralRole {
    fn is_boundary_signal_role(&self) -> bool {
        matches!(
            self,
            Self::Boundary | Self::Coboundary | Self::Incidence | Self::Composition
        )
    }
}

/// Finite observation about one subject in a generic structural role.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralObservation {
    /// Stable observation identifier.
    pub id: Id,
    /// Stable identifier of the observed subject.
    pub subject_id: Id,
    /// Generic role carried by the observation.
    pub role: StructuralRole,
    /// Source identifiers that support this observation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

impl StructuralObservation {
    /// Creates an observation with no source identifiers.
    #[must_use]
    pub fn new(id: Id, subject_id: Id, role: StructuralRole) -> Self {
        Self {
            id,
            subject_id,
            role,
            source_ids: Vec::new(),
        }
    }

    /// Returns this observation with one source identifier appended.
    #[must_use]
    pub fn with_source(mut self, source_id: Id) -> Self {
        self.source_ids = stable_unique(self.source_ids.into_iter().chain([source_id]));
        self
    }

    /// Returns this observation with source identifiers appended.
    #[must_use]
    pub fn with_sources(mut self, source_ids: impl IntoIterator<Item = Id>) -> Self {
        self.source_ids = stable_unique(self.source_ids.into_iter().chain(source_ids));
        self
    }
}

/// Generic signal that a subject has finite structural boundary-like evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralBoundarySignal {
    /// Stable identifier of the signaled subject.
    pub subject_id: Id,
    /// Unique structural roles observed for the subject.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<StructuralRole>,
    /// Unique observations contributing to this subject signal.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observation_ids: Vec<Id>,
    /// Unique source identifiers supporting the contributing observations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

/// Analysis result for finite structural boundary-like observations.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralBoundaryAnalysis {
    /// Signals grouped by subject identifier.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signals: Vec<StructuralBoundarySignal>,
}

/// Analyzer for finite generic structural observations.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralBoundaryAnalyzer {
    /// Observations to analyze.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<StructuralObservation>,
}

impl StructuralBoundaryAnalyzer {
    /// Creates an analyzer with no observations.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns this analyzer with one observation appended.
    #[must_use]
    pub fn with_observation(mut self, observation: StructuralObservation) -> Self {
        self.observations.push(observation);
        self
    }

    /// Returns this analyzer with observations appended.
    #[must_use]
    pub fn with_observations(
        mut self,
        observations: impl IntoIterator<Item = StructuralObservation>,
    ) -> Self {
        self.observations.extend(observations);
        self
    }

    /// Groups observations by subject and reports subjects with boundary-like roles.
    #[must_use]
    pub fn analyze(&self) -> StructuralBoundaryAnalysis {
        let mut groups = BTreeMap::<Id, SubjectAccumulator>::new();

        for observation in &self.observations {
            let group = groups.entry(observation.subject_id.clone()).or_default();
            group.roles.insert(observation.role.clone());
            group.observation_ids.insert(observation.id.clone());
            group
                .source_ids
                .extend(observation.source_ids.iter().cloned());
            group.has_boundary_signal_role |= observation.role.is_boundary_signal_role();
        }

        let signals = groups
            .into_iter()
            .filter_map(|(subject_id, group)| {
                group
                    .has_boundary_signal_role
                    .then(|| StructuralBoundarySignal {
                        subject_id,
                        roles: group.roles.into_iter().collect(),
                        observation_ids: ids_from_set(group.observation_ids),
                        source_ids: ids_from_set(group.source_ids),
                    })
            })
            .collect();

        StructuralBoundaryAnalysis { signals }
    }
}

#[derive(Default)]
struct SubjectAccumulator {
    roles: BTreeSet<StructuralRole>,
    observation_ids: BTreeSet<Id>,
    source_ids: BTreeSet<Id>,
    has_boundary_signal_role: bool,
}

fn stable_unique(ids: impl IntoIterator<Item = Id>) -> Vec<Id> {
    ids_from_set(ids.into_iter().collect())
}

fn ids_from_set(ids: BTreeSet<Id>) -> Vec<Id> {
    ids.into_iter().collect()
}
