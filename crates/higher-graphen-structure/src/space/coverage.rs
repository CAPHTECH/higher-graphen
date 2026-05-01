//! Deterministic finite coverage and greedy set-cover helpers.

use higher_graphen_core::Id;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Candidate that can cover a finite set of universe elements.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageCandidate {
    /// Stable candidate identifier.
    pub id: Id,
    /// Universe element identifiers covered by this candidate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covers: Vec<Id>,
    /// Higher priority candidates win ties after uncovered coverage size.
    pub priority: u32,
    /// Lower cost candidates win ties after priority.
    pub cost: u32,
}

impl CoverageCandidate {
    /// Creates a unit-cost candidate with zero priority.
    #[must_use]
    pub fn new(id: Id, covers: impl IntoIterator<Item = Id>) -> Self {
        Self {
            id,
            covers: stable_unique(covers),
            priority: 0,
            cost: 1,
        }
    }

    /// Returns this candidate with a tie-break priority.
    #[must_use]
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Returns this candidate with a non-zero cost. Zero is normalized to one.
    #[must_use]
    pub fn with_cost(mut self, cost: u32) -> Self {
        self.cost = cost.max(1);
        self
    }
}

/// Result of deterministic greedy finite coverage selection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageSelection {
    /// Selected candidate identifiers in deterministic selection order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_ids: Vec<Id>,
    /// Universe elements covered by the selected candidates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_ids: Vec<Id>,
    /// Universe elements left uncovered.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uncovered_ids: Vec<Id>,
}

/// Weighted universe element for finite coverage selection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WeightedUniverseElement {
    /// Stable universe element identifier.
    pub id: Id,
    /// Positive selection weight. Higher weight means covering this element is more valuable.
    pub weight: u32,
}

impl WeightedUniverseElement {
    /// Creates a weighted universe element. Zero is normalized to one.
    #[must_use]
    pub fn new(id: Id, weight: u32) -> Self {
        Self {
            id,
            weight: weight.max(1),
        }
    }
}

/// Result of deterministic greedy weighted finite coverage selection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WeightedCoverageSelection {
    /// Selected candidate identifiers in deterministic selection order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_ids: Vec<Id>,
    /// Universe elements covered by the selected candidates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_ids: Vec<Id>,
    /// Universe elements left uncovered.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uncovered_ids: Vec<Id>,
    /// Sum of covered universe-element weights.
    pub covered_weight: u32,
    /// Sum of uncovered universe-element weights.
    pub uncovered_weight: u32,
}

/// Deterministic greedy set-cover selector over finite identifier sets.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GreedyCoverageSelector {
    /// Universe to cover.
    pub universe: Vec<Id>,
    /// Candidates available to cover the universe.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<CoverageCandidate>,
    /// Optional maximum number of candidates to select.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<usize>,
}

impl GreedyCoverageSelector {
    /// Creates a selector with no budget cap.
    #[must_use]
    pub fn new(universe: impl IntoIterator<Item = Id>) -> Self {
        Self {
            universe: stable_unique(universe),
            candidates: Vec::new(),
            budget: None,
        }
    }

    /// Appends one coverage candidate.
    #[must_use]
    pub fn with_candidate(mut self, candidate: CoverageCandidate) -> Self {
        self.candidates.push(candidate);
        self
    }

    /// Appends coverage candidates.
    #[must_use]
    pub fn with_candidates(
        mut self,
        candidates: impl IntoIterator<Item = CoverageCandidate>,
    ) -> Self {
        self.candidates.extend(candidates);
        self
    }

    /// Adds a candidate budget. Zero means no candidates can be selected.
    #[must_use]
    pub fn with_budget(mut self, budget: usize) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Selects candidates greedily by uncovered coverage, priority, cost, and id.
    #[must_use]
    pub fn select(&self) -> CoverageSelection {
        let weighted = WeightedCoverageSelector::from_ids(self.universe.iter().cloned())
            .with_candidates(self.candidates.clone());
        let weighted = if let Some(budget) = self.budget {
            weighted.with_budget(budget)
        } else {
            weighted
        };
        let selection = weighted.select();

        CoverageSelection {
            selected_ids: selection.selected_ids,
            covered_ids: selection.covered_ids,
            uncovered_ids: selection.uncovered_ids,
        }
    }
}

/// Deterministic greedy set-cover selector over a weighted finite universe.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WeightedCoverageSelector {
    /// Weighted universe to cover.
    pub universe: Vec<WeightedUniverseElement>,
    /// Candidates available to cover the universe.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<CoverageCandidate>,
    /// Optional maximum number of candidates to select.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<usize>,
}

impl WeightedCoverageSelector {
    /// Creates a selector with no budget cap.
    #[must_use]
    pub fn new(universe: impl IntoIterator<Item = WeightedUniverseElement>) -> Self {
        Self {
            universe: stable_unique_weighted(universe),
            candidates: Vec::new(),
            budget: None,
        }
    }

    /// Creates a unit-weight selector from universe identifiers.
    #[must_use]
    pub fn from_ids(universe: impl IntoIterator<Item = Id>) -> Self {
        Self::new(
            stable_unique(universe)
                .into_iter()
                .map(|id| WeightedUniverseElement::new(id, 1)),
        )
    }

    /// Appends one coverage candidate.
    #[must_use]
    pub fn with_candidate(mut self, candidate: CoverageCandidate) -> Self {
        self.candidates.push(candidate);
        self
    }

    /// Appends coverage candidates.
    #[must_use]
    pub fn with_candidates(
        mut self,
        candidates: impl IntoIterator<Item = CoverageCandidate>,
    ) -> Self {
        self.candidates.extend(candidates);
        self
    }

    /// Adds a candidate budget. Zero means no candidates can be selected.
    #[must_use]
    pub fn with_budget(mut self, budget: usize) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Selects candidates greedily by uncovered weight, priority, cost, and id.
    #[must_use]
    pub fn select(&self) -> WeightedCoverageSelection {
        let weights = weight_map(self.universe.iter().cloned());
        let mut uncovered = weights.keys().cloned().collect::<BTreeSet<_>>();
        let mut covered = BTreeSet::<Id>::new();
        let mut selected_ids = Vec::<Id>::new();
        let mut selected_candidate_ids = BTreeSet::<Id>::new();
        let budget = self.budget.unwrap_or(usize::MAX);

        while !uncovered.is_empty() && selected_ids.len() < budget {
            let Some(candidate) =
                self.best_candidate(&weights, &uncovered, &selected_candidate_ids)
            else {
                break;
            };
            selected_candidate_ids.insert(candidate.id.clone());
            selected_ids.push(candidate.id.clone());
            for covered_id in candidate
                .covers
                .iter()
                .filter(|id| weights.contains_key(*id))
            {
                covered.insert(covered_id.clone());
                uncovered.remove(covered_id);
            }
        }

        let covered_weight = sum_weight(&covered, &weights);
        let uncovered_weight = sum_weight(&uncovered, &weights);

        WeightedCoverageSelection {
            selected_ids,
            covered_ids: ids_from_set(covered),
            uncovered_ids: ids_from_set(uncovered),
            covered_weight,
            uncovered_weight,
        }
    }

    fn best_candidate<'a>(
        &'a self,
        weights: &BTreeMap<Id, u32>,
        uncovered: &BTreeSet<Id>,
        selected_candidate_ids: &BTreeSet<Id>,
    ) -> Option<&'a CoverageCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| !selected_candidate_ids.contains(&candidate.id))
            .filter_map(|candidate| {
                let uncovered_weight = candidate
                    .covers
                    .iter()
                    .filter(|covered_id| uncovered.contains(*covered_id))
                    .filter_map(|covered_id| weights.get(covered_id))
                    .copied()
                    .sum::<u32>();
                (uncovered_weight > 0).then_some((candidate, uncovered_weight))
            })
            .max_by(|(left, left_weight), (right, right_weight)| {
                left_weight
                    .cmp(right_weight)
                    .then_with(|| left.priority.cmp(&right.priority))
                    .then_with(|| right.cost.cmp(&left.cost))
                    .then_with(|| right.id.cmp(&left.id))
            })
            .map(|(candidate, _)| candidate)
    }
}

/// A deterministic dominance relation between two finite coverage candidates.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DominanceRelation {
    /// Candidate that dominates another candidate.
    pub dominant_id: Id,
    /// Candidate whose coverage and selection profile are no better than the dominant candidate.
    pub dominated_id: Id,
    /// Elements that make the domination comparable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_ids: Vec<Id>,
}

/// Deterministic report of candidate dominance relations.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DominanceReport {
    /// Pairwise dominance relations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<DominanceRelation>,
    /// Candidate identifiers dominated by at least one other candidate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dominated_ids: Vec<Id>,
}

/// Analyzer for dominance between finite coverage candidates.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DominanceAnalysis {
    /// Candidates to compare.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<CoverageCandidate>,
}

impl DominanceAnalysis {
    /// Creates an analyzer from candidates.
    #[must_use]
    pub fn new(candidates: impl IntoIterator<Item = CoverageCandidate>) -> Self {
        Self {
            candidates: candidates.into_iter().collect(),
        }
    }

    /// Computes deterministic pairwise dominance relations.
    #[must_use]
    pub fn analyze(&self) -> DominanceReport {
        let mut relations = Vec::<DominanceRelation>::new();
        let mut dominated_ids = BTreeSet::<Id>::new();

        for left in &self.candidates {
            for right in &self.candidates {
                if left.id == right.id {
                    continue;
                }
                if dominates(left, right) {
                    dominated_ids.insert(right.id.clone());
                    relations.push(DominanceRelation {
                        dominant_id: left.id.clone(),
                        dominated_id: right.id.clone(),
                        covered_ids: stable_unique(right.covers.iter().cloned()),
                    });
                }
            }
        }

        relations.sort_by(|left, right| {
            left.dominant_id
                .cmp(&right.dominant_id)
                .then_with(|| left.dominated_id.cmp(&right.dominated_id))
        });

        DominanceReport {
            relations,
            dominated_ids: ids_from_set(dominated_ids),
        }
    }
}

fn dominates(left: &CoverageCandidate, right: &CoverageCandidate) -> bool {
    let left_covers = id_set(left.covers.iter().cloned());
    let right_covers = id_set(right.covers.iter().cloned());
    if right_covers.is_empty() || !left_covers.is_superset(&right_covers) {
        return false;
    }

    let no_worse = left.priority >= right.priority && left.cost <= right.cost;
    if !no_worse {
        return false;
    }

    left_covers != right_covers
        || left.priority > right.priority
        || left.cost < right.cost
        || left.id < right.id
}

fn stable_unique_weighted(
    elements: impl IntoIterator<Item = WeightedUniverseElement>,
) -> Vec<WeightedUniverseElement> {
    weight_map(elements)
        .into_iter()
        .map(|(id, weight)| WeightedUniverseElement { id, weight })
        .collect()
}

fn weight_map(elements: impl IntoIterator<Item = WeightedUniverseElement>) -> BTreeMap<Id, u32> {
    let mut weights = BTreeMap::<Id, u32>::new();
    for element in elements {
        weights
            .entry(element.id)
            .and_modify(|weight| *weight = (*weight).max(element.weight.max(1)))
            .or_insert(element.weight.max(1));
    }
    weights
}

fn sum_weight(ids: &BTreeSet<Id>, weights: &BTreeMap<Id, u32>) -> u32 {
    ids.iter()
        .filter_map(|id| weights.get(id))
        .copied()
        .sum::<u32>()
}

fn stable_unique(ids: impl IntoIterator<Item = Id>) -> Vec<Id> {
    ids_from_set(id_set(ids))
}

fn id_set(ids: impl IntoIterator<Item = Id>) -> BTreeSet<Id> {
    ids.into_iter().collect()
}

fn ids_from_set(ids: BTreeSet<Id>) -> Vec<Id> {
    ids.into_iter().collect()
}
