//! Contract tests for semantic proof certificate verification.

use higher_graphen_core::{Confidence, Id, ReviewStatus, Severity, SourceKind};
use higher_graphen_runtime::{
    run_semantic_proof_verify, SemanticProofCell, SemanticProofCertificate,
    SemanticProofCounterexample, SemanticProofInputDocument, SemanticProofLaw,
    SemanticProofMorphism, SemanticProofSource, SemanticProofStatus, SemanticProofTheorem,
    SemanticProofVerificationPolicy,
};

#[test]
fn accepted_certificate_proves_theorem() {
    let report = run_semantic_proof_verify(proved_fixture()).expect("workflow should run");

    assert_eq!(report.schema, "highergraphen.semantic_proof.report.v1");
    assert_eq!(report.report_type, "semantic_proof");
    assert_eq!(report.result.status, SemanticProofStatus::Proved);
    assert_eq!(
        report.result.accepted_certificate_ids,
        vec![id("certificate:semantic:pricing")]
    );
    assert!(report.result.issues.is_empty());
    assert!(report.result.proof_objects.iter().any(|proof| {
        proof
            .morphism_ids
            .contains(&id("morphism:semantic:pricing-signature"))
            && proof.review_status == ReviewStatus::Accepted
    }));
}

#[test]
fn missing_certificate_reports_insufficient_proof() {
    let mut input = proved_fixture();
    input.proof_certificates.clear();

    let report = run_semantic_proof_verify(input).expect("workflow should run");

    assert_eq!(report.result.status, SemanticProofStatus::InsufficientProof);
    assert!(report
        .result
        .issues
        .iter()
        .any(|issue| issue.issue_type == "missing_morphism_proof"));
}

#[test]
fn counterexample_takes_precedence_over_proof() {
    let mut input = proved_fixture();
    input.counterexamples.push(SemanticProofCounterexample {
        id: id("counterexample:semantic:pricing"),
        counterexample_type: "symbolic_execution_model".to_owned(),
        theorem_id: id("theorem:semantic:pricing"),
        law_ids: vec![id("law:semantic:signature-preserved")],
        morphism_ids: vec![id("morphism:semantic:pricing-signature")],
        path_ids: vec![
            id("cell:semantic:pricing:base"),
            id("cell:semantic:pricing:head"),
        ],
        summary: "Symbolic execution found a signature preservation counterexample.".to_owned(),
        severity: Severity::Critical,
        confidence: confidence(0.93),
        review_status: ReviewStatus::Unreviewed,
    });

    let report = run_semantic_proof_verify(input).expect("workflow should run");

    assert_eq!(
        report.result.status,
        SemanticProofStatus::CounterexampleFound
    );
    assert_eq!(report.result.counterexamples.len(), 1);
    assert!(report
        .projection
        .human_review
        .recommended_actions
        .iter()
        .any(|action| action.contains("counterexample")));
}

fn proved_fixture() -> SemanticProofInputDocument {
    SemanticProofInputDocument {
        schema: "highergraphen.semantic_proof.input.v1".to_owned(),
        source: SemanticProofSource {
            kind: SourceKind::Code,
            uri: Some("git:fixture".to_owned()),
            title: Some("semantic proof fixture".to_owned()),
            confidence: confidence(1.0),
            adapters: vec!["semantic-proof-fixture.v1".to_owned()],
        },
        theorem: SemanticProofTheorem {
            id: id("theorem:semantic:pricing"),
            summary: "Pricing typed signature is preserved.".to_owned(),
            law_ids: vec![id("law:semantic:signature-preserved")],
            morphism_ids: vec![id("morphism:semantic:pricing-signature")],
        },
        semantic_cells: vec![
            SemanticProofCell {
                id: id("cell:semantic:pricing:base"),
                cell_type: "mir_function".to_owned(),
                label: "base calculate_discount MIR".to_owned(),
                source_ids: Vec::new(),
                confidence: Some(confidence(0.9)),
            },
            SemanticProofCell {
                id: id("cell:semantic:pricing:head"),
                cell_type: "mir_function".to_owned(),
                label: "head calculate_discount MIR".to_owned(),
                source_ids: Vec::new(),
                confidence: Some(confidence(0.9)),
            },
        ],
        morphisms: vec![SemanticProofMorphism {
            id: id("morphism:semantic:pricing-signature"),
            morphism_type: "typed_signature_preservation".to_owned(),
            source_ids: vec![id("cell:semantic:pricing:base")],
            target_ids: vec![id("cell:semantic:pricing:head")],
            law_ids: vec![id("law:semantic:signature-preserved")],
            confidence: Some(confidence(0.86)),
        }],
        laws: vec![SemanticProofLaw {
            id: id("law:semantic:signature-preserved"),
            summary: "Public typed signature is preserved.".to_owned(),
            applies_to_ids: vec![id("morphism:semantic:pricing-signature")],
            confidence: Some(confidence(0.88)),
        }],
        proof_certificates: vec![SemanticProofCertificate {
            id: id("certificate:semantic:pricing"),
            certificate_type: "formal_proof".to_owned(),
            backend: "kani".to_owned(),
            backend_version: "1.0.0".to_owned(),
            theorem_id: id("theorem:semantic:pricing"),
            law_ids: vec![id("law:semantic:signature-preserved")],
            morphism_ids: vec![id("morphism:semantic:pricing-signature")],
            witness_ids: vec![
                id("cell:semantic:pricing:base"),
                id("cell:semantic:pricing:head"),
            ],
            input_hash: Some("sha256:input".to_owned()),
            proof_hash: Some("sha256:proof".to_owned()),
            confidence: confidence(0.91),
            review_status: ReviewStatus::Accepted,
        }],
        counterexamples: Vec::new(),
        verification_policy: SemanticProofVerificationPolicy {
            accepted_backends: vec!["kani".to_owned(), "prusti".to_owned()],
            require_input_hash: true,
            require_proof_hash: true,
            require_accepted_review: true,
        },
    }
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id should be valid")
}

fn confidence(value: f64) -> Confidence {
    Confidence::new(value).expect("test confidence should be valid")
}
