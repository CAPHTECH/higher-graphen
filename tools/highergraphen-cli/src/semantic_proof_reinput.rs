//! Rebuilds semantic proof inputs from insufficient proof reports.

use higher_graphen_core::{Id, SourceKind};
use higher_graphen_runtime::{
    SemanticProofCell, SemanticProofInputDocument, SemanticProofMorphism, SemanticProofReport,
    SemanticProofSource, SemanticProofStatus,
};

const ADAPTER_NAME: &str = "semantic-proof-reinput-from-report.v1";

pub(crate) fn input_from_report(
    report: SemanticProofReport,
) -> Result<SemanticProofInputDocument, String> {
    if report.result.status != SemanticProofStatus::InsufficientProof {
        return Err(format!(
            "semantic proof report status {:?} has no proof obligations to reinput",
            report.result.status
        ));
    }

    let scenario = report.scenario;
    let mut law_ids = Vec::new();
    let mut morphism_ids = Vec::new();
    for issue in report.result.issues {
        match issue.issue_type.as_str() {
            "missing_law_proof" => push_ids(&mut law_ids, issue.target_ids),
            "missing_morphism_proof" => push_ids(&mut morphism_ids, issue.target_ids),
            _ => {}
        }
    }

    if law_ids.is_empty() && morphism_ids.is_empty() {
        law_ids = scenario.theorem.law_ids.clone();
        morphism_ids = scenario.theorem.morphism_ids.clone();
    }
    if law_ids.is_empty() && morphism_ids.is_empty() {
        return Err(
            "semantic proof report contains no reinputable law or morphism obligations".to_owned(),
        );
    }

    for morphism in &scenario.morphisms {
        if morphism_ids.contains(&morphism.id) {
            push_ids(&mut law_ids, morphism.law_ids.clone());
        }
    }
    for law in &scenario.laws {
        if law_ids.contains(&law.id) {
            push_ids(&mut morphism_ids, law.applies_to_ids.clone());
        }
    }

    let morphisms = scenario
        .morphisms
        .into_iter()
        .filter(|morphism| morphism_ids.contains(&morphism.id))
        .collect::<Vec<_>>();
    let laws = scenario
        .laws
        .into_iter()
        .filter(|law| law_ids.contains(&law.id))
        .collect::<Vec<_>>();
    let semantic_cells = cells_for_morphisms(scenario.semantic_cells, &morphisms);
    let mut theorem = scenario.theorem;
    theorem.law_ids = laws.iter().map(|law| law.id.clone()).collect();
    theorem.morphism_ids = morphisms
        .iter()
        .map(|morphism| morphism.id.clone())
        .collect();

    Ok(SemanticProofInputDocument {
        schema: "highergraphen.semantic_proof.input.v1".to_owned(),
        source: reinput_source(scenario.source, theorem.id.as_str()),
        theorem,
        semantic_cells,
        morphisms,
        laws,
        proof_certificates: Vec::new(),
        counterexamples: Vec::new(),
        verification_policy: scenario.verification_policy,
    })
}

fn cells_for_morphisms(
    cells: Vec<SemanticProofCell>,
    morphisms: &[SemanticProofMorphism],
) -> Vec<SemanticProofCell> {
    let mut cell_ids = Vec::new();
    for morphism in morphisms {
        push_ids(&mut cell_ids, morphism.source_ids.clone());
        push_ids(&mut cell_ids, morphism.target_ids.clone());
    }
    cells
        .into_iter()
        .filter(|cell| cell_ids.contains(&cell.id))
        .collect()
}

fn reinput_source(mut source: SemanticProofSource, theorem_id: &str) -> SemanticProofSource {
    if !source
        .adapters
        .iter()
        .any(|adapter| adapter == ADAPTER_NAME)
    {
        source.adapters.push(ADAPTER_NAME.to_owned());
    }
    if source.uri.is_none() {
        source.uri = Some(format!("semantic-proof-reinput:{theorem_id}"));
    }
    if source.title.is_none() {
        source.title = Some("Semantic proof reinput obligations".to_owned());
    }
    source.kind = SourceKind::Code;
    source
}

fn push_ids(target: &mut Vec<Id>, ids: Vec<Id>) {
    for id in ids {
        if !target.contains(&id) {
            target.push(id);
        }
    }
}
