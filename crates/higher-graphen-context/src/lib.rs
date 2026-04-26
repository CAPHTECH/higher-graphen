//! Context, section, restriction, cover, and gluing checks for HigherGraphen.
//!
//! This crate provides a product-neutral local-to-global kernel. A
//! [`Context`] names the elements visible in a local view, a [`Section`] assigns
//! values to those elements, a [`Restriction`] projects a section onto a smaller
//! context, and a [`Cover`] declares which local contexts should cover a base
//! context. [`GluingCheck`] then deterministically reports whether local
//! sections can be merged into one coherent global section.

use higher_graphen_core::{CoreError, Id, Provenance, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Stable assignment map from context element identifiers to product-owned values.
pub type AssignmentMap<V> = BTreeMap<Id, V>;

type ContextIndex<'a> = BTreeMap<Id, &'a Context>;
type SectionIndex<'a, V> = BTreeMap<Id, Vec<&'a Section<V>>>;

/// Named collection of elements that can receive local assignments.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    /// Stable context identifier.
    pub id: Id,
    /// Human-readable context name.
    pub name: String,
    /// Optional context description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Elements visible inside this context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub element_ids: Vec<Id>,
    /// Source and review metadata for this context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl Context {
    /// Creates a context with a required non-empty name.
    pub fn new(id: Id, name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            id,
            name: required_text("name", name)?,
            description: None,
            element_ids: Vec::new(),
            provenance: None,
        })
    }

    /// Returns this context with an optional description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into().trim().to_owned());
        self
    }

    /// Returns this context with one element included.
    #[must_use]
    pub fn with_element(mut self, element_id: Id) -> Self {
        push_unique_sorted(&mut self.element_ids, element_id);
        self
    }

    /// Returns this context with all supplied elements included.
    #[must_use]
    pub fn with_elements<I>(mut self, element_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        self.element_ids = unique_sorted_ids(self.element_ids.into_iter().chain(element_ids));
        self
    }

    /// Returns this context with source and review metadata.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Returns true when the element belongs to this context.
    #[must_use]
    pub fn contains_element(&self, element_id: &Id) -> bool {
        self.element_ids.contains(element_id)
    }
}

/// Assignment of product-owned values over one context.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "V: Deserialize<'de>", serialize = "V: Serialize"))]
pub struct Section<V> {
    /// Stable section identifier.
    pub id: Id,
    /// Context over which this section is defined.
    pub context_id: Id,
    /// Values assigned to context elements.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub assignments: AssignmentMap<V>,
    /// Source and review metadata for this section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl<V> Section<V> {
    /// Creates an empty section over a context.
    #[must_use]
    pub fn new(id: Id, context_id: Id) -> Self {
        Self {
            id,
            context_id,
            assignments: BTreeMap::new(),
            provenance: None,
        }
    }

    /// Returns this section with one assignment inserted or replaced.
    #[must_use]
    pub fn with_assignment(mut self, element_id: Id, value: V) -> Self {
        self.assignments.insert(element_id, value);
        self
    }

    /// Returns this section with source and review metadata.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}

impl<V> Section<V>
where
    V: Clone,
{
    /// Builds a section over `target_context_id` by keeping selected assignments.
    #[must_use]
    pub fn restrict_to_elements<I>(
        &self,
        output_section_id: Id,
        target_context_id: Id,
        element_ids: I,
    ) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        let assignments = unique_sorted_ids(element_ids)
            .into_iter()
            .filter_map(|element_id| {
                self.assignments
                    .get(&element_id)
                    .cloned()
                    .map(|value| (element_id, value))
            })
            .collect();

        Self {
            id: output_section_id,
            context_id: target_context_id,
            assignments,
            provenance: self.provenance.clone(),
        }
    }
}

/// Product-neutral projection from one context to a smaller overlap context.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Restriction {
    /// Stable restriction identifier.
    pub id: Id,
    /// Context from which assignments are read.
    pub source_context_id: Id,
    /// Context to which assignments are projected.
    pub target_context_id: Id,
    /// Elements retained by the restriction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub element_ids: Vec<Id>,
    /// Source and review metadata for this restriction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl Restriction {
    /// Creates a restriction that keeps the supplied element identifiers.
    #[must_use]
    pub fn new<I>(id: Id, source_context_id: Id, target_context_id: Id, element_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        Self {
            id,
            source_context_id,
            target_context_id,
            element_ids: unique_sorted_ids(element_ids),
            provenance: None,
        }
    }

    /// Returns this restriction with source and review metadata.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Applies this restriction to a section over the source context.
    pub fn apply<V>(&self, output_section_id: Id, section: &Section<V>) -> Result<Section<V>>
    where
        V: Clone,
    {
        if section.context_id != self.source_context_id {
            return Err(malformed(
                "section.context_id",
                format!(
                    "section {} belongs to context {}, expected {}",
                    section.id, section.context_id, self.source_context_id
                ),
            ));
        }

        Ok(section.restrict_to_elements(
            output_section_id,
            self.target_context_id.clone(),
            self.element_ids.clone(),
        ))
    }
}

/// Declaration that local member contexts should cover a base context.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Cover {
    /// Stable cover identifier.
    pub id: Id,
    /// Context that should be reconstructed from local sections.
    pub base_context_id: Id,
    /// Local context identifiers that jointly cover the base context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub member_context_ids: Vec<Id>,
    /// Source and review metadata for this cover.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl Cover {
    /// Creates a cover over a base context.
    #[must_use]
    pub fn new<I>(id: Id, base_context_id: Id, member_context_ids: I) -> Self
    where
        I: IntoIterator<Item = Id>,
    {
        Self {
            id,
            base_context_id,
            member_context_ids: unique_sorted_ids(member_context_ids),
            provenance: None,
        }
    }

    /// Returns this cover with one member included.
    #[must_use]
    pub fn with_member(mut self, context_id: Id) -> Self {
        push_unique_sorted(&mut self.member_context_ids, context_id);
        self
    }

    /// Returns this cover with source and review metadata.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}

/// Role a context plays when a gluing check reports that it is missing.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextRole {
    /// The base context named by the cover is missing.
    Base,
    /// A local cover member context is missing.
    CoverMember,
    /// A restriction source context is missing.
    RestrictionSource,
    /// A restriction target context is missing.
    RestrictionTarget,
}

/// Stable high-level result state for a gluing check.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GluingStatus {
    /// Local sections were compatible and produced a global section.
    Gluable,
    /// At least one missing member, malformed boundary, or overlap conflict was found.
    NotGluable,
}

/// Concrete witness for two local assignments that disagree on an overlap.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "V: Deserialize<'de>", serialize = "V: Serialize"))]
pub struct OverlapConflict<V> {
    /// First cover member context.
    pub left_context_id: Id,
    /// Second cover member context.
    pub right_context_id: Id,
    /// First local section.
    pub left_section_id: Id,
    /// Second local section.
    pub right_section_id: Id,
    /// Shared element whose assignments disagree.
    pub element_id: Id,
    /// Value assigned by the first local section.
    pub left_value: V,
    /// Value assigned by the second local section.
    pub right_value: V,
}

/// Structured reason why local sections could not be glued.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[serde(bound(deserialize = "V: Deserialize<'de>", serialize = "V: Serialize"))]
pub enum GluingObstruction<V> {
    /// A context referenced by the check is absent.
    MissingContext {
        /// Missing context identifier.
        context_id: Id,
        /// Role the missing context plays in the check.
        role: ContextRole,
    },
    /// A cover member has no local section.
    MissingCoverMember {
        /// Context identifier for the uncovered local member.
        context_id: Id,
    },
    /// More than one local section was supplied for a cover member.
    DuplicateLocalSection {
        /// Context with multiple local sections.
        context_id: Id,
        /// Section identifiers supplied for that context.
        section_ids: Vec<Id>,
    },
    /// A local section belongs to a context outside the cover.
    SectionOutsideCover {
        /// Section outside the active cover.
        section_id: Id,
        /// Context named by the section.
        context_id: Id,
    },
    /// A section assigns an element not contained in its context.
    AssignmentOutsideContext {
        /// Section containing the extra assignment.
        section_id: Id,
        /// Context over which the section is defined.
        context_id: Id,
        /// Element assigned outside the context.
        element_id: Id,
    },
    /// A local section does not assign a value required by its context.
    MissingLocalAssignment {
        /// Section missing the assignment.
        section_id: Id,
        /// Context over which the section is defined.
        context_id: Id,
        /// Unassigned context element.
        element_id: Id,
    },
    /// A cover member contains an element outside the base context.
    CoverMemberOutsideBase {
        /// Local context containing the extra element.
        member_context_id: Id,
        /// Element outside the base context.
        element_id: Id,
    },
    /// No cover member contains a base context element.
    UncoveredBaseElement {
        /// Base element not covered by any member context.
        element_id: Id,
    },
    /// A restriction names an element absent from one of its endpoint contexts.
    RestrictionElementOutsideContext {
        /// Restriction containing the invalid element.
        restriction_id: Id,
        /// Context missing the element.
        context_id: Id,
        /// Element absent from the context.
        element_id: Id,
    },
    /// Two local sections assign different values to the same overlap element.
    IncompatibleOverlap {
        /// Conflict witness that downstream obstruction handlers can inspect.
        witness: OverlapConflict<V>,
    },
}

/// Deterministic result of evaluating a gluing check.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
#[serde(bound(deserialize = "V: Deserialize<'de>", serialize = "V: Serialize"))]
pub enum GluingCheckResult<V> {
    /// Local assignments were compatible and merged into a global section.
    Gluable {
        /// Check that produced the result.
        check_id: Id,
        /// Global section over the cover base context.
        glued_section: Box<Section<V>>,
        /// Number of pairwise overlap assignments compared.
        checked_overlap_count: usize,
    },
    /// The check found one or more obstructions.
    NotGluable {
        /// Check that produced the result.
        check_id: Id,
        /// Structured obstructions found during evaluation.
        obstructions: Vec<GluingObstruction<V>>,
        /// Number of pairwise overlap assignments compared.
        checked_overlap_count: usize,
    },
}

impl<V> GluingCheckResult<V> {
    /// Returns the high-level result status.
    #[must_use]
    pub fn status(&self) -> GluingStatus {
        match self {
            Self::Gluable { .. } => GluingStatus::Gluable,
            Self::NotGluable { .. } => GluingStatus::NotGluable,
        }
    }

    /// Returns true when the check produced a glued section.
    #[must_use]
    pub fn is_gluable(&self) -> bool {
        self.status() == GluingStatus::Gluable
    }
}

mod check;
pub use check::GluingCheck;

fn index_contexts(contexts: &[Context]) -> ContextIndex<'_> {
    contexts
        .iter()
        .map(|context| (context.id.clone(), context))
        .collect()
}

fn index_sections<V>(sections: &[Section<V>]) -> SectionIndex<'_, V> {
    let mut sections_by_context: SectionIndex<'_, V> = BTreeMap::new();
    for section in sections {
        sections_by_context
            .entry(section.context_id.clone())
            .or_default()
            .push(section);
    }
    sections_by_context
}

fn single_section<'a, V>(
    sections_by_context: &'a SectionIndex<'a, V>,
    context_id: &Id,
) -> Option<&'a Section<V>> {
    sections_by_context
        .get(context_id)
        .and_then(|sections| (sections.len() == 1).then_some(sections[0]))
}

fn overlap_ids(left: &[Id], right: &[Id]) -> Vec<Id> {
    let right = id_set(right.iter().cloned());
    left.iter()
        .filter(|element_id| right.contains(*element_id))
        .cloned()
        .collect()
}

fn id_set(ids: impl IntoIterator<Item = Id>) -> BTreeSet<Id> {
    ids.into_iter().collect()
}

fn unique_sorted_ids(ids: impl IntoIterator<Item = Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn push_unique_sorted(ids: &mut Vec<Id>, id: Id) {
    ids.push(id);
    *ids = unique_sorted_ids(std::mem::take(ids));
}

fn required_text(field: &str, value: impl Into<String>) -> Result<String> {
    let normalized = value.into().trim().to_owned();
    if normalized.is_empty() {
        Err(malformed(field, "value must not be empty after trimming"))
    } else {
        Ok(normalized)
    }
}

fn malformed(field: &str, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
