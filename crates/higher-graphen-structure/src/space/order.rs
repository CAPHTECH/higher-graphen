//! Finite partial-order and lattice-candidate analysis.

use crate::morphism::Morphism;
use higher_graphen_core::{CoreError, Id, Provenance, Result, ReviewStatus};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Reviewable order relation between two elements in one space.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrderRelation {
    /// Stable relation identifier.
    pub id: Id,
    /// Space that owns both ordered elements.
    pub space_id: Id,
    /// Product-neutral relation type, such as refinement, support, or strength.
    pub relation_type: String,
    /// Lesser element in the order relation.
    pub lesser_id: Id,
    /// Greater element in the order relation.
    pub greater_id: Id,
    /// Criteria used to justify this relation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<String>,
    /// Human or workflow review status.
    #[serde(default)]
    pub review_status: ReviewStatus,
    /// Optional source metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl OrderRelation {
    /// Creates a relation with validated type text.
    pub fn new(
        id: Id,
        space_id: Id,
        relation_type: impl Into<String>,
        lesser_id: Id,
        greater_id: Id,
    ) -> Result<Self> {
        Ok(Self {
            id,
            space_id,
            relation_type: required_text("order_relation.relation_type", relation_type)?,
            lesser_id,
            greater_id,
            criteria: Vec::new(),
            review_status: ReviewStatus::Unreviewed,
            provenance: None,
        })
    }

    /// Returns this relation with normalized criteria.
    pub fn with_criteria(mut self, criteria: impl IntoIterator<Item = String>) -> Result<Self> {
        self.criteria = criteria
            .into_iter()
            .map(|criterion| required_text("order_relation.criteria", criterion))
            .collect::<Result<Vec<_>>>()?;
        self.criteria = unique_text(std::mem::take(&mut self.criteria));
        Ok(self)
    }

    /// Returns this relation with review status.
    #[must_use]
    pub fn with_review_status(mut self, review_status: ReviewStatus) -> Self {
        self.review_status = review_status;
        self
    }

    /// Returns this relation with provenance.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}

/// Finite selected relation set for one order type.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FiniteOrderRelationSet {
    /// Stable relation-set identifier.
    pub id: Id,
    /// Space analyzed.
    pub space_id: Id,
    /// Product-neutral relation type selected for this set.
    pub relation_type: String,
    /// Declared elements in the finite carrier.
    pub element_ids: Vec<Id>,
    /// Explicit selected order relations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<OrderRelation>,
}

impl FiniteOrderRelationSet {
    /// Creates an empty finite order relation set.
    pub fn new(
        id: Id,
        space_id: Id,
        relation_type: impl Into<String>,
        element_ids: impl IntoIterator<Item = Id>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            space_id,
            relation_type: required_text("order_relation_set.relation_type", relation_type)?,
            element_ids: unique_ids(element_ids),
            relations: Vec::new(),
        })
    }

    /// Returns this set with one relation appended.
    #[must_use]
    pub fn with_relation(mut self, relation: OrderRelation) -> Self {
        self.relations.push(relation);
        self
    }

    /// Returns this set with relations appended.
    #[must_use]
    pub fn with_relations(mut self, relations: impl IntoIterator<Item = OrderRelation>) -> Self {
        self.relations.extend(relations);
        self
    }

    /// Returns a relation set containing only relations with accepted review status.
    #[must_use]
    pub fn accepted_relations(&self) -> Self {
        self.selected_by_review_statuses([ReviewStatus::Accepted])
    }

    /// Returns a relation set containing only relations whose review status is selected.
    #[must_use]
    pub fn selected_by_review_statuses(
        &self,
        statuses: impl IntoIterator<Item = ReviewStatus>,
    ) -> Self {
        let statuses = statuses.into_iter().collect::<BTreeSet<_>>();
        let relations = self
            .relations
            .iter()
            .filter(|relation| statuses.contains(&relation.review_status))
            .cloned()
            .collect();
        Self {
            id: self.id.clone(),
            space_id: self.space_id.clone(),
            relation_type: self.relation_type.clone(),
            element_ids: self.element_ids.clone(),
            relations,
        }
    }

    /// Runs finite partial-order and bound-candidate analysis.
    pub fn analyze(&self) -> Result<OrderCheckReport> {
        let closure = OrderClosure::build(self)?;
        let antisymmetry_violations = antisymmetry_violations(&closure);
        let incomparable_pairs = incomparable_pairs(&closure);
        let least_upper_bound_candidates = bound_candidates(&closure, BoundKind::LeastUpper);
        let greatest_lower_bound_candidates = bound_candidates(&closure, BoundKind::GreatestLower);
        let mut obstructions = Vec::new();

        for violation in &antisymmetry_violations {
            obstructions.push(OrderObstruction {
                obstruction_type: OrderObstructionType::AntisymmetryViolation,
                reason: format!(
                    "elements {} and {} are mutually reachable",
                    violation.lesser_id, violation.greater_id
                ),
            });
        }
        for candidates in &least_upper_bound_candidates {
            if candidates.candidate_ids.len() > 1 {
                obstructions.push(OrderObstruction {
                    obstruction_type: OrderObstructionType::JoinNotUnique,
                    reason: format!(
                        "elements {} and {} have multiple least upper bound candidates",
                        candidates.left_id, candidates.right_id
                    ),
                });
            }
        }
        for candidates in &greatest_lower_bound_candidates {
            if candidates.candidate_ids.len() > 1 {
                obstructions.push(OrderObstruction {
                    obstruction_type: OrderObstructionType::MeetNotUnique,
                    reason: format!(
                        "elements {} and {} have multiple greatest lower bound candidates",
                        candidates.left_id, candidates.right_id
                    ),
                });
            }
        }

        Ok(OrderCheckReport {
            relation_set_id: self.id.clone(),
            space_id: self.space_id.clone(),
            relation_type: self.relation_type.clone(),
            selected_relation_ids: self
                .relations
                .iter()
                .map(|relation| relation.id.clone())
                .collect(),
            status: if antisymmetry_violations.is_empty() {
                OrderCheckStatus::PartialOrder
            } else {
                OrderCheckStatus::Invalid
            },
            cycle_witness: antisymmetry_violations.first().map(|violation| {
                vec![
                    violation.lesser_id.clone(),
                    violation.greater_id.clone(),
                    violation.lesser_id.clone(),
                ]
            }),
            antisymmetry_violations,
            incomparable_pairs,
            least_upper_bound_candidates,
            greatest_lower_bound_candidates,
            obstructions,
        })
    }

    /// Returns true when `lesser_id <= greater_id` is implied by explicit relations and reflexivity.
    pub fn implies(&self, lesser_id: &Id, greater_id: &Id) -> Result<bool> {
        Ok(OrderClosure::build(self)?.leq(lesser_id, greater_id))
    }
}

/// Partial-order check status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderCheckStatus {
    /// The selected finite relation is reflexively closed and antisymmetric under transitive reachability.
    PartialOrder,
    /// The selected relation violates partial-order requirements.
    Invalid,
}

/// Stable obstruction category from order analysis.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderObstructionType {
    /// Two distinct elements are mutually reachable.
    AntisymmetryViolation,
    /// At least one pair has more than one least upper bound candidate.
    JoinNotUnique,
    /// At least one pair has more than one greatest lower bound candidate.
    MeetNotUnique,
    /// A morphism failed to preserve an implied order relation.
    MonotonicityViolation,
}

/// Structured order obstruction.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrderObstruction {
    /// Obstruction category.
    pub obstruction_type: OrderObstructionType,
    /// Human-readable diagnostic.
    pub reason: String,
}

/// Pair of elements involved in a relation check.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrderPair {
    /// Left element.
    pub left_id: Id,
    /// Right element.
    pub right_id: Id,
}

/// Antisymmetry violation witness.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AntisymmetryViolation {
    /// First element in the mutually reachable pair.
    pub lesser_id: Id,
    /// Second element in the mutually reachable pair.
    pub greater_id: Id,
}

/// Meet or join candidates for one unordered element pair.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrderBoundCandidates {
    /// First element in identifier order.
    pub left_id: Id,
    /// Second element in identifier order.
    pub right_id: Id,
    /// Candidate bound identifiers.
    pub candidate_ids: Vec<Id>,
}

/// Deterministic finite order analysis report.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrderCheckReport {
    /// Relation set analyzed.
    pub relation_set_id: Id,
    /// Space analyzed.
    pub space_id: Id,
    /// Relation type analyzed.
    pub relation_type: String,
    /// Explicit relation identifiers selected for this analysis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_relation_ids: Vec<Id>,
    /// Partial-order validity status.
    pub status: OrderCheckStatus,
    /// Small cycle witness when antisymmetry fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_witness: Option<Vec<Id>>,
    /// Mutually reachable distinct element pairs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub antisymmetry_violations: Vec<AntisymmetryViolation>,
    /// Pairs with no order relation in either direction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incomparable_pairs: Vec<OrderPair>,
    /// Least upper bound candidates by unordered pair.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub least_upper_bound_candidates: Vec<OrderBoundCandidates>,
    /// Greatest lower bound candidates by unordered pair.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub greatest_lower_bound_candidates: Vec<OrderBoundCandidates>,
    /// Obstructions discovered by the analysis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<OrderObstruction>,
}

/// A failed monotonicity witness for a morphism.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrderMonotonicityViolation {
    /// Source lesser element.
    pub source_lesser_id: Id,
    /// Source greater element.
    pub source_greater_id: Id,
    /// Mapped target lesser element.
    pub target_lesser_id: Id,
    /// Mapped target greater element.
    pub target_greater_id: Id,
}

/// Report for checking whether a morphism preserves selected order relations.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrderMonotonicityReport {
    /// Source relation set.
    pub source_relation_set_id: Id,
    /// Target relation set.
    pub target_relation_set_id: Id,
    /// Morphism checked.
    pub morphism_id: Id,
    /// Source elements not mapped by the morphism.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unmapped_source_ids: Vec<Id>,
    /// Monotonicity violations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub violations: Vec<OrderMonotonicityViolation>,
    /// Obstructions derived from violations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obstructions: Vec<OrderObstruction>,
}

impl OrderMonotonicityReport {
    /// Returns true when every mapped source order relation is preserved.
    pub fn is_monotone(&self) -> bool {
        self.violations.is_empty()
    }
}

/// Checks whether a morphism preserves all implied source order relations.
pub fn check_order_monotonicity(
    source: &FiniteOrderRelationSet,
    target: &FiniteOrderRelationSet,
    morphism: &Morphism,
) -> Result<OrderMonotonicityReport> {
    if source.space_id != morphism.source_space_id {
        return Err(CoreError::MalformedField {
            field: "morphism.source_space_id".to_owned(),
            reason: format!(
                "morphism source space {} does not match source order space {}",
                morphism.source_space_id, source.space_id
            ),
        });
    }
    if target.space_id != morphism.target_space_id {
        return Err(CoreError::MalformedField {
            field: "morphism.target_space_id".to_owned(),
            reason: format!(
                "morphism target space {} does not match target order space {}",
                morphism.target_space_id, target.space_id
            ),
        });
    }

    let source_closure = OrderClosure::build(source)?;
    let target_closure = OrderClosure::build(target)?;
    let mut unmapped_source_ids = BTreeSet::new();
    let mut violations = Vec::new();

    for lesser_id in &source_closure.elements {
        for greater_id in &source_closure.elements {
            if lesser_id == greater_id || !source_closure.leq(lesser_id, greater_id) {
                continue;
            }
            let Some(target_lesser_id) = morphism.cell_mapping.get(lesser_id) else {
                unmapped_source_ids.insert(lesser_id.clone());
                continue;
            };
            let Some(target_greater_id) = morphism.cell_mapping.get(greater_id) else {
                unmapped_source_ids.insert(greater_id.clone());
                continue;
            };
            if !target_closure.leq(target_lesser_id, target_greater_id) {
                violations.push(OrderMonotonicityViolation {
                    source_lesser_id: lesser_id.clone(),
                    source_greater_id: greater_id.clone(),
                    target_lesser_id: target_lesser_id.clone(),
                    target_greater_id: target_greater_id.clone(),
                });
            }
        }
    }

    let obstructions = violations
        .iter()
        .map(|violation| OrderObstruction {
            obstruction_type: OrderObstructionType::MonotonicityViolation,
            reason: format!(
                "source order {} <= {} maps to target pair {} <= {} which is not implied",
                violation.source_lesser_id,
                violation.source_greater_id,
                violation.target_lesser_id,
                violation.target_greater_id
            ),
        })
        .collect();

    Ok(OrderMonotonicityReport {
        source_relation_set_id: source.id.clone(),
        target_relation_set_id: target.id.clone(),
        morphism_id: morphism.id.clone(),
        unmapped_source_ids: unmapped_source_ids.into_iter().collect(),
        violations,
        obstructions,
    })
}

#[derive(Clone, Debug)]
struct OrderClosure {
    elements: Vec<Id>,
    reachable: BTreeMap<Id, BTreeSet<Id>>,
}

impl OrderClosure {
    fn build(set: &FiniteOrderRelationSet) -> Result<Self> {
        let relation_type = required_text("order_relation_set.relation_type", &set.relation_type)?;
        let mut elements = set.element_ids.iter().cloned().collect::<BTreeSet<_>>();
        let mut adjacency = BTreeMap::<Id, BTreeSet<Id>>::new();

        for relation in &set.relations {
            if relation.space_id != set.space_id {
                return Err(CoreError::MalformedField {
                    field: "order_relation.space_id".to_owned(),
                    reason: format!(
                        "relation {} belongs to space {}, expected {}",
                        relation.id, relation.space_id, set.space_id
                    ),
                });
            }
            if relation.relation_type != relation_type {
                return Err(CoreError::MalformedField {
                    field: "order_relation.relation_type".to_owned(),
                    reason: format!(
                        "relation {} has type {}, expected {}",
                        relation.id, relation.relation_type, relation_type
                    ),
                });
            }
            elements.insert(relation.lesser_id.clone());
            elements.insert(relation.greater_id.clone());
            adjacency
                .entry(relation.lesser_id.clone())
                .or_default()
                .insert(relation.greater_id.clone());
        }

        let elements = elements.into_iter().collect::<Vec<_>>();
        for element_id in &elements {
            adjacency
                .entry(element_id.clone())
                .or_default()
                .insert(element_id.clone());
        }

        let reachable = elements
            .iter()
            .map(|element_id| (element_id.clone(), reachable_from(element_id, &adjacency)))
            .collect();

        Ok(Self {
            elements,
            reachable,
        })
    }

    fn leq(&self, lesser_id: &Id, greater_id: &Id) -> bool {
        self.reachable
            .get(lesser_id)
            .is_some_and(|reachable| reachable.contains(greater_id))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoundKind {
    LeastUpper,
    GreatestLower,
}

fn reachable_from(start: &Id, adjacency: &BTreeMap<Id, BTreeSet<Id>>) -> BTreeSet<Id> {
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(element_id) = queue.pop_front() {
        if !visited.insert(element_id.clone()) {
            continue;
        }
        if let Some(next_ids) = adjacency.get(&element_id) {
            queue.extend(next_ids.iter().cloned());
        }
    }
    visited
}

fn antisymmetry_violations(closure: &OrderClosure) -> Vec<AntisymmetryViolation> {
    unordered_pairs(&closure.elements)
        .into_iter()
        .filter(|pair| {
            closure.leq(&pair.left_id, &pair.right_id) && closure.leq(&pair.right_id, &pair.left_id)
        })
        .map(|pair| AntisymmetryViolation {
            lesser_id: pair.left_id,
            greater_id: pair.right_id,
        })
        .collect()
}

fn incomparable_pairs(closure: &OrderClosure) -> Vec<OrderPair> {
    unordered_pairs(&closure.elements)
        .into_iter()
        .filter(|pair| {
            !closure.leq(&pair.left_id, &pair.right_id)
                && !closure.leq(&pair.right_id, &pair.left_id)
        })
        .collect()
}

fn bound_candidates(closure: &OrderClosure, kind: BoundKind) -> Vec<OrderBoundCandidates> {
    unordered_pairs(&closure.elements)
        .into_iter()
        .filter_map(|pair| {
            let candidate_ids = match kind {
                BoundKind::LeastUpper => least_upper_bounds(closure, &pair.left_id, &pair.right_id),
                BoundKind::GreatestLower => {
                    greatest_lower_bounds(closure, &pair.left_id, &pair.right_id)
                }
            };
            (!candidate_ids.is_empty()).then_some(OrderBoundCandidates {
                left_id: pair.left_id,
                right_id: pair.right_id,
                candidate_ids,
            })
        })
        .collect()
}

fn least_upper_bounds(closure: &OrderClosure, left_id: &Id, right_id: &Id) -> Vec<Id> {
    let upper_bounds = closure
        .elements
        .iter()
        .filter(|candidate| closure.leq(left_id, candidate) && closure.leq(right_id, candidate))
        .cloned()
        .collect::<Vec<_>>();
    upper_bounds
        .iter()
        .filter(|candidate| {
            !upper_bounds
                .iter()
                .any(|other| other != *candidate && closure.leq(other, candidate))
        })
        .cloned()
        .collect()
}

fn greatest_lower_bounds(closure: &OrderClosure, left_id: &Id, right_id: &Id) -> Vec<Id> {
    let lower_bounds = closure
        .elements
        .iter()
        .filter(|candidate| closure.leq(candidate, left_id) && closure.leq(candidate, right_id))
        .cloned()
        .collect::<Vec<_>>();
    lower_bounds
        .iter()
        .filter(|candidate| {
            !lower_bounds
                .iter()
                .any(|other| other != *candidate && closure.leq(candidate, other))
        })
        .cloned()
        .collect()
}

fn unordered_pairs(elements: &[Id]) -> Vec<OrderPair> {
    let mut pairs = Vec::new();
    for (left_index, left_id) in elements.iter().enumerate() {
        for right_id in elements.iter().skip(left_index + 1) {
            pairs.push(OrderPair {
                left_id: left_id.clone(),
                right_id: right_id.clone(),
            });
        }
    }
    pairs
}

fn required_text(field: &'static str, value: impl Into<String>) -> Result<String> {
    let normalized = value.into().trim().to_owned();
    if normalized.is_empty() {
        Err(CoreError::MalformedField {
            field: field.to_owned(),
            reason: "value must not be empty after trimming".to_owned(),
        })
    } else {
        Ok(normalized)
    }
}

fn unique_ids(ids: impl IntoIterator<Item = Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn unique_text(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        check_order_monotonicity, FiniteOrderRelationSet, OrderBoundCandidates, OrderCheckReport,
        OrderCheckStatus, OrderMonotonicityReport, OrderObstructionType, OrderPair, OrderRelation,
    };
    use crate::morphism::{Morphism, MorphismType};
    use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, SourceKind, SourceRef};
    use std::collections::BTreeMap;

    fn id(value: &str) -> Id {
        Id::new(value).expect("valid id")
    }

    #[test]
    fn partial_order_reports_incomparables_and_unique_bounds() {
        let set = order_set(
            "order/lattice",
            "space/order",
            ["bottom", "a", "b", "top"],
            [
                ("bottom-to-a", "bottom", "a"),
                ("bottom-to-b", "bottom", "b"),
                ("a-to-top", "a", "top"),
                ("b-to-top", "b", "top"),
            ],
        );

        let report = set.analyze().expect("analyze order");

        assert_eq!(report.status, OrderCheckStatus::PartialOrder);
        assert_eq!(
            report.incomparable_pairs,
            vec![OrderPair {
                left_id: id("a"),
                right_id: id("b"),
            }]
        );
        assert!(report.obstructions.is_empty());
        assert!(set.implies(&id("bottom"), &id("top")).expect("closure"));
        assert_eq!(
            bound_for(&report.least_upper_bound_candidates, "a", "b"),
            vec![id("top")]
        );
        assert_eq!(
            bound_for(&report.greatest_lower_bound_candidates, "a", "b"),
            vec![id("bottom")]
        );

        let roundtrip: OrderCheckReport =
            serde_json::from_str(&serde_json::to_string(&report).expect("serialize"))
                .expect("deserialize");
        assert_eq!(roundtrip, report);
    }

    #[test]
    fn antisymmetry_violation_marks_order_invalid() {
        let set = order_set(
            "order/cycle",
            "space/order",
            ["a", "b"],
            [("a-to-b", "a", "b"), ("b-to-a", "b", "a")],
        );

        let report = set.analyze().expect("analyze order");

        assert_eq!(report.status, OrderCheckStatus::Invalid);
        assert_eq!(report.cycle_witness, Some(vec![id("a"), id("b"), id("a")]));
        assert_eq!(
            report.obstructions[0].obstruction_type,
            OrderObstructionType::AntisymmetryViolation
        );
    }

    #[test]
    fn accepted_relation_filter_limits_order_analysis_input() {
        let accepted = OrderRelation::new(
            id("accepted"),
            id("space/order"),
            "refines",
            id("a"),
            id("b"),
        )
        .expect("relation")
        .with_review_status(ReviewStatus::Accepted);
        let unreviewed = OrderRelation::new(
            id("unreviewed"),
            id("space/order"),
            "refines",
            id("b"),
            id("a"),
        )
        .expect("relation");
        let set = FiniteOrderRelationSet::new(
            id("order/reviewed"),
            id("space/order"),
            "refines",
            [id("a"), id("b")],
        )
        .expect("set")
        .with_relations([accepted, unreviewed]);

        let report = set.accepted_relations().analyze().expect("analyze");

        assert_eq!(report.status, OrderCheckStatus::PartialOrder);
        assert_eq!(report.selected_relation_ids, vec![id("accepted")]);
        assert!(report.antisymmetry_violations.is_empty());
    }

    #[test]
    fn non_unique_join_and_meet_are_reported_as_obstructions() {
        let set = order_set(
            "order/non-unique",
            "space/order",
            ["left", "right", "upper-a", "upper-b", "lower-a", "lower-b"],
            [
                ("left-to-upper-a", "left", "upper-a"),
                ("right-to-upper-a", "right", "upper-a"),
                ("left-to-upper-b", "left", "upper-b"),
                ("right-to-upper-b", "right", "upper-b"),
                ("lower-a-to-left", "lower-a", "left"),
                ("lower-a-to-right", "lower-a", "right"),
                ("lower-b-to-left", "lower-b", "left"),
                ("lower-b-to-right", "lower-b", "right"),
            ],
        );

        let report = set.analyze().expect("analyze order");

        assert_eq!(
            bound_for(&report.least_upper_bound_candidates, "left", "right"),
            vec![id("upper-a"), id("upper-b")]
        );
        assert_eq!(
            bound_for(&report.greatest_lower_bound_candidates, "left", "right"),
            vec![id("lower-a"), id("lower-b")]
        );
        assert!(
            report
                .obstructions
                .iter()
                .any(|obstruction| obstruction.obstruction_type
                    == OrderObstructionType::JoinNotUnique)
        );
        assert!(
            report
                .obstructions
                .iter()
                .any(|obstruction| obstruction.obstruction_type
                    == OrderObstructionType::MeetNotUnique)
        );
    }

    #[test]
    fn monotonicity_check_reports_unpreserved_order_relation() {
        let source = order_set(
            "order/source",
            "space/source",
            ["a", "b"],
            [("a-to-b", "a", "b")],
        );
        let target = order_set("order/target", "space/target", ["x", "y"], []);
        let morphism = morphism([("a", "x"), ("b", "y")]);

        let report =
            check_order_monotonicity(&source, &target, &morphism).expect("check monotonicity");

        assert!(!report.is_monotone());
        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.obstructions[0].obstruction_type,
            OrderObstructionType::MonotonicityViolation
        );

        let ordered_target = order_set(
            "order/target",
            "space/target",
            ["x", "y"],
            [("x-to-y", "x", "y")],
        );
        let monotone =
            check_order_monotonicity(&source, &ordered_target, &morphism).expect("check monotone");
        assert!(monotone.is_monotone());
        assert!(monotone.obstructions.is_empty());

        let roundtrip: OrderMonotonicityReport =
            serde_json::from_str(&serde_json::to_string(&monotone).expect("serialize"))
                .expect("deserialize");
        assert_eq!(roundtrip, monotone);
    }

    fn order_set<const E: usize, const R: usize>(
        set_id: &str,
        space_id: &str,
        elements: [&str; E],
        relations: [(&str, &str, &str); R],
    ) -> FiniteOrderRelationSet {
        FiniteOrderRelationSet::new(
            id(set_id),
            id(space_id),
            "refines",
            elements.into_iter().map(id),
        )
        .expect("order set")
        .with_relations(relations.into_iter().map(
            |(relation_id, lesser_id, greater_id)| {
                OrderRelation::new(
                    id(relation_id),
                    id(space_id),
                    "refines",
                    id(lesser_id),
                    id(greater_id),
                )
                .expect("relation")
                .with_review_status(ReviewStatus::Accepted)
            },
        ))
    }

    fn bound_for(candidates: &[OrderBoundCandidates], left: &str, right: &str) -> Vec<Id> {
        candidates
            .iter()
            .find(|candidate| candidate.left_id == id(left) && candidate.right_id == id(right))
            .map(|candidate| candidate.candidate_ids.clone())
            .unwrap_or_default()
    }

    fn morphism<const N: usize>(cell_pairs: [(&str, &str); N]) -> Morphism {
        Morphism {
            id: id("morphism/source-target"),
            source_space_id: id("space/source"),
            target_space_id: id("space/target"),
            name: "source to target".to_owned(),
            morphism_type: MorphismType::Translation,
            cell_mapping: cell_pairs
                .into_iter()
                .map(|(source, target)| (id(source), id(target)))
                .collect::<BTreeMap<_, _>>(),
            relation_mapping: BTreeMap::new(),
            preserved_invariant_ids: Vec::new(),
            lost_structure: Vec::new(),
            distortion: Vec::new(),
            composable_with: Vec::new(),
            provenance: Provenance::new(
                SourceRef::new(SourceKind::custom("order-test").expect("source kind")),
                Confidence::ONE,
            )
            .with_review_status(ReviewStatus::Accepted),
        }
    }
}
