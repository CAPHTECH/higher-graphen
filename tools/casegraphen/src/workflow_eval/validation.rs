use crate::workflow_model::{
    CompletionReviewAction, CorrespondenceType, EvidenceBoundary, InformationLoss,
    WorkflowCaseGraph, WorkflowProvenance, WorkflowRelationType, WORKFLOW_GRAPH_SCHEMA,
    WORKFLOW_GRAPH_SCHEMA_VERSION,
};
use higher_graphen_core::Id;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub type WorkflowResult<T> = Result<T, WorkflowValidationError>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowValidationError {
    pub violations: Vec<WorkflowValidationViolation>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowValidationViolation {
    pub code: WorkflowValidationCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_id: Option<Id>,
    pub field: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowValidationCode {
    SchemaMismatch,
    UnsupportedSchemaVersion,
    DuplicateId,
    EmptyRequiredField,
    SpaceMismatch,
    DanglingReference,
    MissingEvidenceSource,
    ReviewStatusMismatch,
    TransitionGraphMismatch,
    CorrespondenceWitnessMissing,
    ProjectionLossMismatch,
}

pub fn validate_workflow_graph(graph: &WorkflowCaseGraph) -> WorkflowResult<()> {
    let mut accum = ValidationAccum::default();
    validate_schema(graph, &mut accum);
    let index = WorkflowIdIndex::new(graph, &mut accum);
    validate_work_items(graph, &index, &mut accum);
    validate_relations(graph, &index, &mut accum);
    validate_readiness_rules(graph, &index, &mut accum);
    validate_evidence_records(graph, &mut accum);
    validate_completion_reviews(graph, &index, &mut accum);
    validate_transition_records(graph, &mut accum);
    validate_projection_profiles(graph, &index, &mut accum);
    validate_correspondence_records(graph, &index, &mut accum);
    accum.finish()
}

#[derive(Default)]
struct ValidationAccum {
    violations: Vec<WorkflowValidationViolation>,
}

impl ValidationAccum {
    fn push(
        &mut self,
        code: WorkflowValidationCode,
        record_id: Option<&Id>,
        field: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.violations.push(WorkflowValidationViolation {
            code,
            record_id: record_id.cloned(),
            field: field.into(),
            message: message.into(),
        });
    }

    fn finish(self) -> WorkflowResult<()> {
        if self.violations.is_empty() {
            Ok(())
        } else {
            Err(WorkflowValidationError {
                violations: self.violations,
            })
        }
    }
}

struct WorkflowIdIndex {
    record_ids: BTreeSet<Id>,
    work_item_ids: BTreeSet<Id>,
    evidence_ids: BTreeSet<Id>,
}

impl WorkflowIdIndex {
    fn new(graph: &WorkflowCaseGraph, accum: &mut ValidationAccum) -> Self {
        let mut typed_ids = BTreeMap::<Id, &'static str>::new();
        let mut work_item_ids = BTreeSet::new();
        let mut evidence_ids = BTreeSet::new();

        for item in &graph.work_items {
            insert_record_id(&mut typed_ids, accum, &item.id, "work_item");
            work_item_ids.insert(item.id.clone());
        }
        for relation in &graph.workflow_relations {
            insert_record_id(&mut typed_ids, accum, &relation.id, "workflow_relation");
        }
        for rule in &graph.readiness_rules {
            insert_record_id(&mut typed_ids, accum, &rule.id, "readiness_rule");
        }
        for evidence in &graph.evidence_records {
            insert_record_id(&mut typed_ids, accum, &evidence.id, "evidence_record");
            evidence_ids.insert(evidence.id.clone());
        }
        for review in &graph.completion_reviews {
            insert_record_id(&mut typed_ids, accum, &review.id, "completion_review");
        }
        for transition in &graph.transition_records {
            insert_record_id(&mut typed_ids, accum, &transition.id, "transition_record");
        }
        for profile in &graph.projection_profiles {
            insert_record_id(&mut typed_ids, accum, &profile.id, "projection_profile");
        }
        for correspondence in &graph.correspondence_records {
            insert_record_id(
                &mut typed_ids,
                accum,
                &correspondence.id,
                "correspondence_record",
            );
        }

        Self {
            record_ids: typed_ids.into_keys().collect(),
            work_item_ids,
            evidence_ids,
        }
    }

    fn has_record(&self, id: &Id) -> bool {
        self.record_ids.contains(id)
    }
}

fn insert_record_id(
    typed_ids: &mut BTreeMap<Id, &'static str>,
    accum: &mut ValidationAccum,
    id: &Id,
    record_type: &'static str,
) {
    if let Some(existing_type) = typed_ids.insert(id.clone(), record_type) {
        accum.push(
            WorkflowValidationCode::DuplicateId,
            Some(id),
            "id",
            format!(
                "duplicate workflow record id {id} appears as both {existing_type} and {record_type}"
            ),
        );
    }
}

fn validate_schema(graph: &WorkflowCaseGraph, accum: &mut ValidationAccum) {
    if graph.schema != WORKFLOW_GRAPH_SCHEMA {
        accum.push(
            WorkflowValidationCode::SchemaMismatch,
            Some(&graph.workflow_graph_id),
            "schema",
            format!(
                "unsupported workflow schema {:?}; expected {:?}",
                graph.schema, WORKFLOW_GRAPH_SCHEMA
            ),
        );
    }
    if graph.schema_version != WORKFLOW_GRAPH_SCHEMA_VERSION {
        accum.push(
            WorkflowValidationCode::UnsupportedSchemaVersion,
            Some(&graph.workflow_graph_id),
            "schema_version",
            format!(
                "unsupported workflow schema version {}; expected {}",
                graph.schema_version, WORKFLOW_GRAPH_SCHEMA_VERSION
            ),
        );
    }
}

fn validate_work_items(
    graph: &WorkflowCaseGraph,
    index: &WorkflowIdIndex,
    accum: &mut ValidationAccum,
) {
    for item in &graph.work_items {
        require_non_empty(accum, &item.id, "title", &item.title);
        validate_provenance(accum, &item.id, &item.provenance);
        if item.space_id != graph.space_id {
            accum.push(
                WorkflowValidationCode::SpaceMismatch,
                Some(&item.id),
                "space_id",
                format!(
                    "work item {} belongs to {}, but workflow graph space_id is {}",
                    item.id, item.space_id, graph.space_id
                ),
            );
        }
        for dependency_id in &item.hard_dependency_ids {
            require_work_item_reference(
                accum,
                index,
                &item.id,
                "hard_dependency_ids",
                dependency_id,
            );
        }
        for proof_id in &item.proof_requirement_ids {
            if internal_work_item_like_id(proof_id) {
                require_work_item_reference(
                    accum,
                    index,
                    &item.id,
                    "proof_requirement_ids",
                    proof_id,
                );
            }
        }
    }
}

fn validate_relations(
    graph: &WorkflowCaseGraph,
    index: &WorkflowIdIndex,
    accum: &mut ValidationAccum,
) {
    for relation in &graph.workflow_relations {
        validate_provenance(accum, &relation.id, &relation.provenance);
        require_internal_endpoint(
            accum,
            index,
            &relation.id,
            "from_id",
            &relation.from_id,
            relation.relation_type,
        );
        if relation.relation_type != WorkflowRelationType::RequiresEvidence {
            require_internal_endpoint(
                accum,
                index,
                &relation.id,
                "to_id",
                &relation.to_id,
                relation.relation_type,
            );
        }
        for evidence_id in &relation.evidence_ids {
            require_evidence_reference(accum, index, &relation.id, "evidence_ids", evidence_id);
        }
    }
}

fn validate_readiness_rules(
    graph: &WorkflowCaseGraph,
    index: &WorkflowIdIndex,
    accum: &mut ValidationAccum,
) {
    for rule in &graph.readiness_rules {
        require_non_empty(accum, &rule.id, "obstruction_type", &rule.obstruction_type);
        validate_provenance(accum, &rule.id, &rule.provenance);
        for target_item_id in &rule.target_item_ids {
            require_work_item_reference(accum, index, &rule.id, "target_item_ids", target_item_id);
        }
    }
}

fn validate_evidence_records(graph: &WorkflowCaseGraph, accum: &mut ValidationAccum) {
    for evidence in &graph.evidence_records {
        require_non_empty(accum, &evidence.id, "summary", &evidence.summary);
        validate_provenance(accum, &evidence.id, &evidence.provenance);
        if matches!(
            evidence.evidence_boundary,
            EvidenceBoundary::AcceptedEvidence | EvidenceBoundary::SourceBackedEvidence
        ) && evidence.source_ids.is_empty()
        {
            accum.push(
                WorkflowValidationCode::MissingEvidenceSource,
                Some(&evidence.id),
                "source_ids",
                format!(
                    "{} is accepted or source-backed evidence but has no source_ids",
                    evidence.id
                ),
            );
        }
    }
}

fn validate_completion_reviews(
    graph: &WorkflowCaseGraph,
    index: &WorkflowIdIndex,
    accum: &mut ValidationAccum,
) {
    for review in &graph.completion_reviews {
        require_non_empty(accum, &review.id, "reason", &review.reason);
        validate_provenance(accum, &review.id, &review.provenance);
        let expected_status = review.action.outcome_review_status();
        if review.outcome_review_status != expected_status {
            accum.push(
                WorkflowValidationCode::ReviewStatusMismatch,
                Some(&review.id),
                "outcome_review_status",
                format!(
                    "{:?} review {} must have outcome_review_status {:?}",
                    review.action, review.id, expected_status
                ),
            );
        }
        if review.provenance.review_status != expected_status
            && review.action != CompletionReviewAction::Reopen
        {
            accum.push(
                WorkflowValidationCode::ReviewStatusMismatch,
                Some(&review.id),
                "provenance.review_status",
                format!(
                    "{} provenance review_status must match explicit outcome {:?}",
                    review.id, expected_status
                ),
            );
        }
        for evidence_id in &review.evidence_ids {
            require_evidence_reference(accum, index, &review.id, "evidence_ids", evidence_id);
        }
        for decision_id in &review.decision_ids {
            require_known_internal_reference(accum, index, &review.id, "decision_ids", decision_id);
        }
    }
}

fn validate_transition_records(graph: &WorkflowCaseGraph, accum: &mut ValidationAccum) {
    for transition in &graph.transition_records {
        validate_provenance(accum, &transition.id, &transition.provenance);
        if transition.source_workflow_graph_id != graph.workflow_graph_id
            && transition.target_workflow_graph_id != graph.workflow_graph_id
        {
            accum.push(
                WorkflowValidationCode::TransitionGraphMismatch,
                Some(&transition.id),
                "source_workflow_graph_id",
                format!(
                    "{} references neither the current workflow graph {} as source nor target",
                    transition.id, graph.workflow_graph_id
                ),
            );
        }
    }
}

fn validate_projection_profiles(
    graph: &WorkflowCaseGraph,
    index: &WorkflowIdIndex,
    accum: &mut ValidationAccum,
) {
    for profile in &graph.projection_profiles {
        require_non_empty(accum, &profile.id, "purpose", &profile.purpose);
        validate_provenance(accum, &profile.id, &profile.provenance);
        for included_id in &profile.included_ids {
            require_known_internal_reference(
                accum,
                index,
                &profile.id,
                "included_ids",
                included_id,
            );
        }
        for loss in &profile.information_loss {
            validate_information_loss(accum, index, &profile.id, loss);
        }
    }
}

fn validate_correspondence_records(
    graph: &WorkflowCaseGraph,
    index: &WorkflowIdIndex,
    accum: &mut ValidationAccum,
) {
    for correspondence in &graph.correspondence_records {
        validate_provenance(accum, &correspondence.id, &correspondence.provenance);
        for evidence_id in &correspondence.mismatch_evidence_ids {
            require_evidence_reference(
                accum,
                index,
                &correspondence.id,
                "mismatch_evidence_ids",
                evidence_id,
            );
        }
        if matches!(
            correspondence.correspondence_type,
            CorrespondenceType::SimilarWithLoss | CorrespondenceType::Conflicting
        ) && correspondence.mismatch_evidence_ids.is_empty()
        {
            accum.push(
                WorkflowValidationCode::CorrespondenceWitnessMissing,
                Some(&correspondence.id),
                "mismatch_evidence_ids",
                format!(
                    "{:?} correspondence {} must retain mismatch evidence ids",
                    correspondence.correspondence_type, correspondence.id
                ),
            );
        }
    }
}

fn validate_information_loss(
    accum: &mut ValidationAccum,
    index: &WorkflowIdIndex,
    profile_id: &Id,
    loss: &InformationLoss,
) {
    require_non_empty(
        accum,
        profile_id,
        "information_loss.description",
        &loss.description,
    );
    for represented_id in &loss.represented_ids {
        if internal_reference_like_id(represented_id) && !index.has_record(represented_id) {
            accum.push(
                WorkflowValidationCode::ProjectionLossMismatch,
                Some(profile_id),
                "information_loss.represented_ids",
                format!("projection loss represented id {represented_id} is not a workflow record"),
            );
        }
    }
}

fn require_internal_endpoint(
    accum: &mut ValidationAccum,
    index: &WorkflowIdIndex,
    relation_id: &Id,
    field: &'static str,
    reference_id: &Id,
    relation_type: WorkflowRelationType,
) {
    if internal_reference_like_id(reference_id) && !index.has_record(reference_id) {
        accum.push(
            WorkflowValidationCode::DanglingReference,
            Some(relation_id),
            field,
            format!(
                "{relation_type:?} relation {relation_id} references missing workflow record {reference_id}"
            ),
        );
    }
}

fn require_known_internal_reference(
    accum: &mut ValidationAccum,
    index: &WorkflowIdIndex,
    record_id: &Id,
    field: &'static str,
    reference_id: &Id,
) {
    if internal_reference_like_id(reference_id) && !index.has_record(reference_id) {
        accum.push(
            WorkflowValidationCode::DanglingReference,
            Some(record_id),
            field,
            format!("{record_id} references missing workflow record {reference_id}"),
        );
    }
}

fn require_work_item_reference(
    accum: &mut ValidationAccum,
    index: &WorkflowIdIndex,
    record_id: &Id,
    field: &'static str,
    reference_id: &Id,
) {
    if !index.work_item_ids.contains(reference_id) {
        accum.push(
            WorkflowValidationCode::DanglingReference,
            Some(record_id),
            field,
            format!("{record_id} references missing work item {reference_id}"),
        );
    }
}

fn require_evidence_reference(
    accum: &mut ValidationAccum,
    index: &WorkflowIdIndex,
    record_id: &Id,
    field: &'static str,
    reference_id: &Id,
) {
    if !index.evidence_ids.contains(reference_id) {
        accum.push(
            WorkflowValidationCode::DanglingReference,
            Some(record_id),
            field,
            format!("{record_id} references missing evidence record {reference_id}"),
        );
    }
}

fn validate_provenance(
    accum: &mut ValidationAccum,
    record_id: &Id,
    provenance: &WorkflowProvenance,
) {
    require_non_empty(
        accum,
        record_id,
        "provenance.source.kind",
        &provenance.source.kind,
    );
}

fn require_non_empty(
    accum: &mut ValidationAccum,
    record_id: &Id,
    field: &'static str,
    value: &str,
) {
    if value.trim().is_empty() {
        accum.push(
            WorkflowValidationCode::EmptyRequiredField,
            Some(record_id),
            field,
            format!("{record_id} has an empty required field {field}"),
        );
    }
}

fn internal_work_item_like_id(id: &Id) -> bool {
    matches!(
        id_prefix(id),
        Some("task")
            | Some("proof")
            | Some("decision")
            | Some("event")
            | Some("external_wait")
            | Some("review_action")
            | Some("milestone")
    )
}

fn internal_reference_like_id(id: &Id) -> bool {
    matches!(
        id_prefix(id),
        Some("task")
            | Some("proof")
            | Some("decision")
            | Some("event")
            | Some("evidence")
            | Some("external_wait")
            | Some("review_action")
            | Some("milestone")
            | Some("relation")
            | Some("readiness")
            | Some("completion_review")
            | Some("transition")
            | Some("projection")
            | Some("correspondence")
    )
}

fn id_prefix(id: &Id) -> Option<&str> {
    id.as_str().split_once(':').map(|(prefix, _)| prefix)
}

impl fmt::Display for WorkflowValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "workflow validation failed with {} violation(s)",
            self.violations.len()
        )?;
        for violation in self.violations.iter().take(3) {
            write!(
                formatter,
                ": {} {} {}",
                violation.code, violation.field, violation.message
            )?;
        }
        if self.violations.len() > 3 {
            write!(formatter, ": ... and {} more", self.violations.len() - 3)?;
        }
        Ok(())
    }
}

impl fmt::Display for WorkflowValidationCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SchemaMismatch => "schema_mismatch",
            Self::UnsupportedSchemaVersion => "unsupported_schema_version",
            Self::DuplicateId => "duplicate_id",
            Self::EmptyRequiredField => "empty_required_field",
            Self::SpaceMismatch => "space_mismatch",
            Self::DanglingReference => "dangling_reference",
            Self::MissingEvidenceSource => "missing_evidence_source",
            Self::ReviewStatusMismatch => "review_status_mismatch",
            Self::TransitionGraphMismatch => "transition_graph_mismatch",
            Self::CorrespondenceWitnessMissing => "correspondence_witness_missing",
            Self::ProjectionLossMismatch => "projection_loss_mismatch",
        })
    }
}

impl std::error::Error for WorkflowValidationError {}
