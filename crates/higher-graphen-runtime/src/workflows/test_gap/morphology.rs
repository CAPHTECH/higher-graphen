use super::*;

pub(super) fn morphism_summaries(
    input: &TestGapInputDocument,
    candidates: &[TestGapCompletionCandidate],
) -> RuntimeResult<Vec<TestGapMorphismSummary>> {
    let mut summaries = Vec::new();
    append_requirement_morphisms(input, &mut summaries)?;
    append_symbol_morphisms(input, &mut summaries)?;
    append_native_morphisms(input, &mut summaries);
    summaries.push(before_to_after_morphism(input)?);
    append_candidate_morphisms(candidates, &mut summaries)?;
    Ok(summaries)
}

fn append_requirement_morphisms(
    input: &TestGapInputDocument,
    summaries: &mut Vec<TestGapMorphismSummary>,
) -> RuntimeResult<()> {
    for requirement in &input.requirements {
        let has_impl = !requirement.implementation_ids.is_empty();
        let has_unit_test = has_accepted_test_for_requirement(input, &requirement.id);
        let mut loss = Vec::new();
        if !has_impl {
            loss.push("requirement has no supplied implementation target".to_owned());
        }
        if !has_unit_test {
            loss.push("requirement has no accepted unit-test verification".to_owned());
        }
        summaries.push(TestGapMorphismSummary {
            id: id(format!(
                "morphism:test-gap:requirement-to-implementation:{}",
                slug(&requirement.id)
            ))?,
            morphism_type: TestGapMorphismType::RequirementToImplementation,
            source_ids: vec![requirement.id.clone()],
            target_ids: requirement.implementation_ids.clone(),
            preservation_status: if loss.is_empty() {
                TestGapPreservationStatus::Preserved
            } else if has_impl {
                TestGapPreservationStatus::Partial
            } else {
                TestGapPreservationStatus::Lost
            },
            preserved: if has_impl {
                vec!["requirement identity and implementation target IDs".to_owned()]
            } else {
                Vec::new()
            },
            loss,
            review_status: ReviewStatus::Accepted,
        });
    }
    Ok(())
}

fn append_symbol_morphisms(
    input: &TestGapInputDocument,
    summaries: &mut Vec<TestGapMorphismSummary>,
) -> RuntimeResult<()> {
    for symbol in &input.symbols {
        let has_unit_test = has_accepted_test_for_symbol(input, &symbol.id);
        summaries.push(TestGapMorphismSummary {
            id: id(format!(
                "morphism:test-gap:implementation-to-test:{}",
                slug(&symbol.id)
            ))?,
            morphism_type: TestGapMorphismType::ImplementationToTest,
            source_ids: vec![symbol.id.clone()],
            target_ids: related_test_ids_for_symbol(input, &symbol.id),
            preservation_status: if has_unit_test {
                TestGapPreservationStatus::Preserved
            } else {
                TestGapPreservationStatus::Lost
            },
            preserved: if has_unit_test {
                vec!["implementation target has accepted unit-test relation".to_owned()]
            } else {
                Vec::new()
            },
            loss: if has_unit_test {
                Vec::new()
            } else {
                vec!["implementation target has no accepted unit-test relation".to_owned()]
            },
            review_status: ReviewStatus::Accepted,
        });
    }
    Ok(())
}

fn append_native_morphisms(
    input: &TestGapInputDocument,
    summaries: &mut Vec<TestGapMorphismSummary>,
) {
    for morphism in &input.morphisms {
        let has_verification = has_accepted_verification_for_morphism(input, &morphism.id);
        summaries.push(TestGapMorphismSummary {
            id: morphism.id.clone(),
            morphism_type: TestGapMorphismType::RequirementToImplementation,
            source_ids: morphism.source_ids.clone(),
            target_ids: morphism.target_ids.clone(),
            preservation_status: if has_verification {
                TestGapPreservationStatus::Preserved
            } else {
                TestGapPreservationStatus::Lost
            },
            preserved: if has_verification {
                vec![format!(
                    "native morphism {} has accepted verification cell",
                    morphism.morphism_type
                )]
            } else {
                Vec::new()
            },
            loss: if has_verification {
                Vec::new()
            } else {
                vec![format!(
                    "native morphism {} has no accepted verification cell",
                    morphism.morphism_type
                )]
            },
            review_status: ReviewStatus::Accepted,
        });
    }
}

fn before_to_after_morphism(input: &TestGapInputDocument) -> RuntimeResult<TestGapMorphismSummary> {
    Ok(TestGapMorphismSummary {
        id: id(format!(
            "morphism:test-gap:before-to-after:{}",
            slug(&input.change_set.id)
        ))?,
        morphism_type: TestGapMorphismType::BeforeToAfter,
        source_ids: input
            .changed_files
            .iter()
            .map(|file| file.id.clone())
            .collect(),
        target_ids: input
            .symbols
            .iter()
            .map(|symbol| symbol.id.clone())
            .collect(),
        preservation_status: TestGapPreservationStatus::Partial,
        preserved: vec!["changed file and symbol identities from bounded input".to_owned()],
        loss: vec!["raw pre-change and post-change source bodies are omitted".to_owned()],
        review_status: ReviewStatus::Accepted,
    })
}

fn append_candidate_morphisms(
    candidates: &[TestGapCompletionCandidate],
    summaries: &mut Vec<TestGapMorphismSummary>,
) -> RuntimeResult<()> {
    for candidate in candidates {
        summaries.push(TestGapMorphismSummary {
            id: id(format!(
                "morphism:test-gap:candidate-to-accepted-test:{}",
                slug(&candidate.id)
            ))?,
            morphism_type: TestGapMorphismType::CandidateToAcceptedTest,
            source_ids: vec![candidate.id.clone()],
            target_ids: Vec::new(),
            preservation_status: TestGapPreservationStatus::NotEvaluated,
            preserved: Vec::new(),
            loss: vec!["candidate remains unreviewed and has no accepted test mapping".to_owned()],
            review_status: ReviewStatus::Unreviewed,
        });
    }
    Ok(())
}

pub(super) fn proof_objects(
    input: &TestGapInputDocument,
) -> RuntimeResult<Vec<TestGapProofObject>> {
    let mut proofs = Vec::new();
    for law in &input.laws {
        let verified_by_ids = accepted_verification_ids_for_law(input, &law.id);
        if verified_by_ids.is_empty() {
            continue;
        }
        let mut witness_ids = vec![law.id.clone()];
        witness_ids.extend(law.applies_to_ids.iter().cloned());
        proofs.push(TestGapProofObject {
            id: id(format!("proof:test-gap:law:{}", slug(&law.id)))?,
            proof_type: "law_verification".to_owned(),
            law_ids: vec![law.id.clone()],
            morphism_ids: Vec::new(),
            verified_by_ids,
            witness_ids,
            summary: format!("Law {} has accepted verification evidence.", law.summary),
            confidence: law.confidence.unwrap_or(Confidence::new(0.82)?),
            review_status: ReviewStatus::Accepted,
        });
    }
    for morphism in &input.morphisms {
        let verified_by_ids = accepted_verification_ids_for_morphism(input, &morphism.id);
        if verified_by_ids.is_empty() {
            continue;
        }
        let mut witness_ids = vec![morphism.id.clone()];
        witness_ids.extend(morphism.source_ids.iter().cloned());
        witness_ids.extend(morphism.target_ids.iter().cloned());
        witness_ids.extend(morphism.law_ids.iter().cloned());
        proofs.push(TestGapProofObject {
            id: id(format!("proof:test-gap:morphism:{}", slug(&morphism.id)))?,
            proof_type: "morphism_verification".to_owned(),
            law_ids: morphism.law_ids.clone(),
            morphism_ids: vec![morphism.id.clone()],
            verified_by_ids,
            witness_ids,
            summary: format!(
                "Morphism {} has accepted verification evidence.",
                morphism.morphism_type
            ),
            confidence: morphism.confidence.unwrap_or(Confidence::new(0.8)?),
            review_status: ReviewStatus::Accepted,
        });
    }
    Ok(proofs)
}

pub(super) fn counterexamples(
    input: &TestGapInputDocument,
) -> RuntimeResult<Vec<TestGapCounterexample>> {
    let mut counterexamples = Vec::new();
    for law in &input.laws {
        if law.expected_verification.is_some()
            && accepted_verification_ids_for_law(input, &law.id).is_empty()
        {
            let mut path_ids = vec![law.id.clone()];
            path_ids.extend(law.applies_to_ids.iter().cloned());
            counterexamples.push(TestGapCounterexample {
                id: id(format!("counterexample:test-gap:law:{}", slug(&law.id)))?,
                counterexample_type: "missing_law_verification".to_owned(),
                law_ids: vec![law.id.clone()],
                morphism_ids: Vec::new(),
                path_ids,
                summary: format!("Law {} has no accepted verification path.", law.summary),
                confidence: law.confidence.unwrap_or(Confidence::new(0.82)?),
                review_status: ReviewStatus::Unreviewed,
            });
        }
    }
    for morphism in &input.morphisms {
        if morphism.expected_verification.is_some()
            && accepted_verification_ids_for_morphism(input, &morphism.id).is_empty()
        {
            let mut path_ids = vec![morphism.id.clone()];
            path_ids.extend(morphism.source_ids.iter().cloned());
            path_ids.extend(morphism.target_ids.iter().cloned());
            path_ids.extend(morphism.law_ids.iter().cloned());
            counterexamples.push(TestGapCounterexample {
                id: id(format!(
                    "counterexample:test-gap:morphism:{}",
                    slug(&morphism.id)
                ))?,
                counterexample_type: "missing_morphism_verification".to_owned(),
                law_ids: morphism.law_ids.clone(),
                morphism_ids: vec![morphism.id.clone()],
                path_ids,
                summary: format!(
                    "Morphism {} has no accepted verification path.",
                    morphism.morphism_type
                ),
                confidence: morphism.confidence.unwrap_or(Confidence::new(0.8)?),
                review_status: ReviewStatus::Unreviewed,
            });
        }
    }
    Ok(counterexamples)
}
