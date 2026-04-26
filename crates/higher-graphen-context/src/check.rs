use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Complete input record for checking whether local sections glue over a cover.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "V: Deserialize<'de>", serialize = "V: Serialize"))]
pub struct GluingCheck<V> {
    /// Stable check identifier.
    pub id: Id,
    /// Cover under evaluation.
    pub cover: Cover,
    /// Section identifier to use when compatible local sections glue.
    pub glued_section_id: Id,
    /// Context definitions referenced by the cover, restrictions, and sections.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<Context>,
    /// Optional reusable restriction declarations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub restrictions: Vec<Restriction>,
    /// Local sections over cover member contexts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_sections: Vec<Section<V>>,
    /// Source and review metadata for this check.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl<V> GluingCheck<V> {
    /// Creates a gluing check with no contexts, restrictions, or sections yet attached.
    #[must_use]
    pub fn new(id: Id, cover: Cover, glued_section_id: Id) -> Self {
        Self {
            id,
            cover,
            glued_section_id,
            contexts: Vec::new(),
            restrictions: Vec::new(),
            local_sections: Vec::new(),
            provenance: None,
        }
    }

    /// Returns this check with one context attached.
    #[must_use]
    pub fn with_context(mut self, context: Context) -> Self {
        self.contexts.push(context);
        self
    }

    /// Returns this check with all supplied contexts attached.
    #[must_use]
    pub fn with_contexts<I>(mut self, contexts: I) -> Self
    where
        I: IntoIterator<Item = Context>,
    {
        self.contexts.extend(contexts);
        self
    }

    /// Returns this check with one restriction attached.
    #[must_use]
    pub fn with_restriction(mut self, restriction: Restriction) -> Self {
        self.restrictions.push(restriction);
        self
    }

    /// Returns this check with one local section attached.
    #[must_use]
    pub fn with_local_section(mut self, section: Section<V>) -> Self {
        self.local_sections.push(section);
        self
    }

    /// Returns this check with all supplied local sections attached.
    #[must_use]
    pub fn with_local_sections<I>(mut self, sections: I) -> Self
    where
        I: IntoIterator<Item = Section<V>>,
    {
        self.local_sections.extend(sections);
        self
    }

    /// Returns this check with source and review metadata.
    #[must_use]
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }
}

impl<V> GluingCheck<V>
where
    V: Clone + Eq,
{
    /// Evaluates local section compatibility and returns a deterministic result.
    #[must_use]
    pub fn evaluate(&self) -> GluingCheckResult<V> {
        let contexts = index_contexts(&self.contexts);
        let cover_member_ids = unique_sorted_ids(self.cover.member_context_ids.clone());
        let cover_member_set = id_set(cover_member_ids.clone());
        let sections_by_context = index_sections(&self.local_sections);
        let mut obstructions = Vec::new();

        let base_context = contexts.get(&self.cover.base_context_id).copied();
        if base_context.is_none() {
            obstructions.push(GluingObstruction::MissingContext {
                context_id: self.cover.base_context_id.clone(),
                role: ContextRole::Base,
            });
        }

        for member_context_id in &cover_member_ids {
            if !contexts.contains_key(member_context_id) {
                obstructions.push(GluingObstruction::MissingContext {
                    context_id: member_context_id.clone(),
                    role: ContextRole::CoverMember,
                });
            }

            match sections_by_context.get(member_context_id) {
                None => obstructions.push(GluingObstruction::MissingCoverMember {
                    context_id: member_context_id.clone(),
                }),
                Some(sections) if sections.len() > 1 => {
                    obstructions.push(GluingObstruction::DuplicateLocalSection {
                        context_id: member_context_id.clone(),
                        section_ids: sections.iter().map(|section| section.id.clone()).collect(),
                    });
                }
                Some(_) => {}
            }
        }

        for section in &self.local_sections {
            if !cover_member_set.contains(&section.context_id) {
                obstructions.push(GluingObstruction::SectionOutsideCover {
                    section_id: section.id.clone(),
                    context_id: section.context_id.clone(),
                });
            }
        }

        self.validate_restrictions(&contexts, &mut obstructions);
        self.validate_cover_shape(
            &contexts,
            base_context,
            &cover_member_ids,
            &mut obstructions,
        );
        self.validate_local_assignments(
            &contexts,
            &cover_member_set,
            &sections_by_context,
            &mut obstructions,
        );

        let checked_overlap_count = self.find_overlap_conflicts(
            &contexts,
            &cover_member_ids,
            &sections_by_context,
            &mut obstructions,
        );

        self.result(obstructions, checked_overlap_count, base_context)
    }

    fn result(
        &self,
        obstructions: Vec<GluingObstruction<V>>,
        checked_overlap_count: usize,
        base_context: Option<&Context>,
    ) -> GluingCheckResult<V> {
        if obstructions.is_empty() {
            GluingCheckResult::Gluable {
                check_id: self.id.clone(),
                glued_section: Box::new(self.build_glued_section(base_context)),
                checked_overlap_count,
            }
        } else {
            GluingCheckResult::NotGluable {
                check_id: self.id.clone(),
                obstructions,
                checked_overlap_count,
            }
        }
    }

    fn validate_restrictions(
        &self,
        contexts: &ContextIndex<'_>,
        obstructions: &mut Vec<GluingObstruction<V>>,
    ) {
        for restriction in &self.restrictions {
            let source = contexts.get(&restriction.source_context_id);
            let target = contexts.get(&restriction.target_context_id);

            if source.is_none() {
                obstructions.push(GluingObstruction::MissingContext {
                    context_id: restriction.source_context_id.clone(),
                    role: ContextRole::RestrictionSource,
                });
            }
            if target.is_none() {
                obstructions.push(GluingObstruction::MissingContext {
                    context_id: restriction.target_context_id.clone(),
                    role: ContextRole::RestrictionTarget,
                });
            }

            for element_id in &restriction.element_ids {
                if let Some(context) = source {
                    if !context.contains_element(element_id) {
                        obstructions.push(GluingObstruction::RestrictionElementOutsideContext {
                            restriction_id: restriction.id.clone(),
                            context_id: context.id.clone(),
                            element_id: element_id.clone(),
                        });
                    }
                }
                if let Some(context) = target {
                    if !context.contains_element(element_id) {
                        obstructions.push(GluingObstruction::RestrictionElementOutsideContext {
                            restriction_id: restriction.id.clone(),
                            context_id: context.id.clone(),
                            element_id: element_id.clone(),
                        });
                    }
                }
            }
        }
    }

    fn validate_cover_shape(
        &self,
        contexts: &ContextIndex<'_>,
        base_context: Option<&Context>,
        cover_member_ids: &[Id],
        obstructions: &mut Vec<GluingObstruction<V>>,
    ) {
        let Some(base_context) = base_context else {
            return;
        };

        let base_elements = id_set(base_context.element_ids.clone());
        let mut covered_elements = BTreeSet::new();

        for member_context_id in cover_member_ids {
            let Some(member_context) = contexts.get(member_context_id) else {
                continue;
            };

            for element_id in &member_context.element_ids {
                if base_elements.contains(element_id) {
                    covered_elements.insert(element_id.clone());
                } else {
                    obstructions.push(GluingObstruction::CoverMemberOutsideBase {
                        member_context_id: member_context.id.clone(),
                        element_id: element_id.clone(),
                    });
                }
            }
        }

        for element_id in &base_elements {
            if !covered_elements.contains(element_id) {
                obstructions.push(GluingObstruction::UncoveredBaseElement {
                    element_id: element_id.clone(),
                });
            }
        }
    }

    fn validate_local_assignments(
        &self,
        contexts: &ContextIndex<'_>,
        cover_member_set: &BTreeSet<Id>,
        sections_by_context: &SectionIndex<'_, V>,
        obstructions: &mut Vec<GluingObstruction<V>>,
    ) {
        for sections in sections_by_context.values() {
            for section in sections {
                if !cover_member_set.contains(&section.context_id) {
                    continue;
                }

                let Some(context) = contexts.get(&section.context_id) else {
                    continue;
                };

                for element_id in section.assignments.keys() {
                    if !context.contains_element(element_id) {
                        obstructions.push(GluingObstruction::AssignmentOutsideContext {
                            section_id: section.id.clone(),
                            context_id: context.id.clone(),
                            element_id: element_id.clone(),
                        });
                    }
                }

                for element_id in &context.element_ids {
                    if !section.assignments.contains_key(element_id) {
                        obstructions.push(GluingObstruction::MissingLocalAssignment {
                            section_id: section.id.clone(),
                            context_id: context.id.clone(),
                            element_id: element_id.clone(),
                        });
                    }
                }
            }
        }
    }

    fn find_overlap_conflicts(
        &self,
        contexts: &ContextIndex<'_>,
        cover_member_ids: &[Id],
        sections_by_context: &SectionIndex<'_, V>,
        obstructions: &mut Vec<GluingObstruction<V>>,
    ) -> usize {
        let mut checked_overlap_count = 0;

        for left_index in 0..cover_member_ids.len() {
            for right_index in (left_index + 1)..cover_member_ids.len() {
                let left_context_id = &cover_member_ids[left_index];
                let right_context_id = &cover_member_ids[right_index];
                let Some(left_context) = contexts.get(left_context_id) else {
                    continue;
                };
                let Some(right_context) = contexts.get(right_context_id) else {
                    continue;
                };
                let Some(left_section) = single_section(sections_by_context, left_context_id)
                else {
                    continue;
                };
                let Some(right_section) = single_section(sections_by_context, right_context_id)
                else {
                    continue;
                };

                for element_id in overlap_ids(&left_context.element_ids, &right_context.element_ids)
                {
                    let Some(left_value) = left_section.assignments.get(&element_id) else {
                        continue;
                    };
                    let Some(right_value) = right_section.assignments.get(&element_id) else {
                        continue;
                    };

                    checked_overlap_count += 1;
                    if left_value != right_value {
                        obstructions.push(GluingObstruction::IncompatibleOverlap {
                            witness: OverlapConflict {
                                left_context_id: left_context.id.clone(),
                                right_context_id: right_context.id.clone(),
                                left_section_id: left_section.id.clone(),
                                right_section_id: right_section.id.clone(),
                                element_id,
                                left_value: left_value.clone(),
                                right_value: right_value.clone(),
                            },
                        });
                    }
                }
            }
        }

        checked_overlap_count
    }

    fn build_glued_section(&self, base_context: Option<&Context>) -> Section<V> {
        let mut assignments = BTreeMap::new();
        let Some(base_context) = base_context else {
            return Section::new(
                self.glued_section_id.clone(),
                self.cover.base_context_id.clone(),
            );
        };
        let base_elements = id_set(base_context.element_ids.clone());

        for section in &self.local_sections {
            for (element_id, value) in &section.assignments {
                if base_elements.contains(element_id) {
                    assignments
                        .entry(element_id.clone())
                        .or_insert_with(|| value.clone());
                }
            }
        }

        Section {
            id: self.glued_section_id.clone(),
            context_id: self.cover.base_context_id.clone(),
            assignments,
            provenance: self.provenance.clone(),
        }
    }
}
