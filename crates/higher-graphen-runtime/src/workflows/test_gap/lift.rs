use super::*;

pub(super) fn lift_input(input: &TestGapInputDocument) -> RuntimeResult<TestGapLiftedStructure> {
    let space_id = space_id(input)?;
    let context_ids = effective_context_ids(input)?;
    let contexts = lifted_contexts(input, &space_id, &context_ids);
    let mut cells = Vec::new();
    append_cells(input, &space_id, &context_ids, &mut cells)?;
    let incidences = lifted_incidences(input, &space_id)?;
    let space = TestGapLiftedSpace {
        id: space_id.clone(),
        name: format!("Test gap space for {}", input.repository.name),
        description: Some(format!(
            "Bounded structural view of {} between {} and {}.",
            input.change_set.boundary, input.change_set.base_ref, input.change_set.head_ref
        )),
        cell_ids: cells.iter().map(|cell| cell.id.clone()).collect(),
        incidence_ids: incidences
            .iter()
            .map(|incidence| incidence.id.clone())
            .collect(),
        context_ids,
    };
    Ok(TestGapLiftedStructure {
        structural_summary: TestGapStructuralSummary {
            accepted_cell_count: cells.len(),
            accepted_incidence_count: incidences.len(),
            context_count: contexts.len(),
            branch_count: input.branches.len(),
            requirement_count: input.requirements.len(),
            test_count: input.tests.len(),
            coverage_record_count: input.coverage.len(),
            higher_order_cell_count: input.higher_order_cells.len(),
            higher_order_incidence_count: input.higher_order_incidences.len(),
            morphism_count: input.morphisms.len(),
            law_count: input.laws.len(),
            verification_cell_count: input.verification_cells.len(),
        },
        space,
        contexts,
        cells,
        incidences,
    })
}

pub(super) fn lifted_contexts(
    input: &TestGapInputDocument,
    space_id: &Id,
    context_ids: &[Id],
) -> Vec<TestGapLiftedContext> {
    context_ids
        .iter()
        .map(|context_id| {
            if let Some(context) = input
                .contexts
                .iter()
                .find(|context| &context.id == context_id)
            {
                TestGapLiftedContext {
                    id: context.id.clone(),
                    space_id: space_id.clone(),
                    name: context.name.clone(),
                    context_type: serde_plain_context_type(context.context_type),
                    provenance: fact_provenance(input, input.source.confidence, Some("contexts"))
                        .expect("valid context provenance"),
                }
            } else {
                TestGapLiftedContext {
                    id: context_id.clone(),
                    space_id: space_id.clone(),
                    name: input.repository.name.clone(),
                    context_type: "repository".to_owned(),
                    provenance: fact_provenance(input, input.source.confidence, Some("repository"))
                        .expect("valid repository provenance"),
                }
            }
        })
        .collect()
}

pub(super) fn append_cells(
    input: &TestGapInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
    cells: &mut Vec<TestGapLiftedCell>,
) -> RuntimeResult<()> {
    append_implementation_cells(input, space_id, default_context_ids, cells)?;
    append_observation_cells(input, space_id, default_context_ids, cells)?;
    append_higher_order_cells(input, space_id, default_context_ids, cells)
}

fn append_implementation_cells(
    input: &TestGapInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
    cells: &mut Vec<TestGapLiftedCell>,
) -> RuntimeResult<()> {
    for file in &input.changed_files {
        cells.push(lifted_cell(
            input,
            space_id,
            file.id.clone(),
            0,
            "test_gap.changed_file",
            file_label(&file.path),
            contexts_or_default(&file.context_ids, default_context_ids),
            input.source.confidence,
            Some("changed_files"),
        )?);
    }
    for symbol in &input.symbols {
        cells.push(lifted_cell(
            input,
            space_id,
            symbol.id.clone(),
            0,
            "test_gap.symbol",
            symbol.name.clone(),
            contexts_or_default(&symbol.context_ids, default_context_ids),
            input.source.confidence,
            Some("symbols"),
        )?);
    }
    for branch in &input.branches {
        cells.push(lifted_cell(
            input,
            space_id,
            branch.id.clone(),
            0,
            "test_gap.branch",
            branch.summary.clone(),
            default_context_ids.to_vec(),
            input.source.confidence,
            Some("branches"),
        )?);
    }
    for requirement in &input.requirements {
        cells.push(lifted_cell(
            input,
            space_id,
            requirement.id.clone(),
            0,
            "test_gap.requirement",
            requirement.summary.clone(),
            default_context_ids.to_vec(),
            input.source.confidence,
            Some("requirements"),
        )?);
    }
    for test in &input.tests {
        cells.push(lifted_cell(
            input,
            space_id,
            test.id.clone(),
            0,
            "test_gap.test",
            test.name.clone(),
            contexts_or_default(&test.context_ids, default_context_ids),
            input.source.confidence,
            Some("tests"),
        )?);
    }
    Ok(())
}

fn append_observation_cells(
    input: &TestGapInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
    cells: &mut Vec<TestGapLiftedCell>,
) -> RuntimeResult<()> {
    for coverage in &input.coverage {
        cells.push(lifted_cell(
            input,
            space_id,
            coverage.id.clone(),
            1,
            "test_gap.coverage",
            coverage
                .summary
                .clone()
                .unwrap_or_else(|| format!("Coverage for {}", coverage.target_id)),
            default_context_ids.to_vec(),
            coverage.confidence.unwrap_or(input.source.confidence),
            Some("coverage"),
        )?);
    }
    for evidence in &input.evidence {
        cells.push(lifted_cell(
            input,
            space_id,
            evidence.id.clone(),
            1,
            "test_gap.evidence",
            evidence.summary.clone(),
            default_context_ids.to_vec(),
            evidence.confidence.unwrap_or(input.source.confidence),
            Some("evidence"),
        )?);
    }
    for signal in &input.signals {
        cells.push(lifted_cell(
            input,
            space_id,
            signal.id.clone(),
            1,
            "test_gap.risk_signal",
            signal.summary.clone(),
            default_context_ids.to_vec(),
            signal.confidence,
            Some("signals"),
        )?);
    }
    Ok(())
}

fn append_higher_order_cells(
    input: &TestGapInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
    cells: &mut Vec<TestGapLiftedCell>,
) -> RuntimeResult<()> {
    for cell in &input.higher_order_cells {
        cells.push(lifted_cell(
            input,
            space_id,
            cell.id.clone(),
            cell.dimension,
            &format!("test_gap.higher_order.{}", cell.cell_type),
            cell.label.clone(),
            contexts_or_default(&cell.context_ids, default_context_ids),
            cell.confidence.unwrap_or(input.source.confidence),
            Some("higher_order_cells"),
        )?);
    }
    for law in &input.laws {
        cells.push(lifted_cell(
            input,
            space_id,
            law.id.clone(),
            1,
            "test_gap.law",
            law.summary.clone(),
            default_context_ids.to_vec(),
            law.confidence.unwrap_or(input.source.confidence),
            Some("laws"),
        )?);
    }
    for morphism in &input.morphisms {
        cells.push(lifted_cell(
            input,
            space_id,
            morphism.id.clone(),
            2,
            "test_gap.morphism",
            morphism.morphism_type.clone(),
            default_context_ids.to_vec(),
            morphism.confidence.unwrap_or(input.source.confidence),
            Some("morphisms"),
        )?);
    }
    for verification in &input.verification_cells {
        cells.push(lifted_cell(
            input,
            space_id,
            verification.id.clone(),
            1,
            &format!("test_gap.verification.{}", verification.verification_type),
            verification.name.clone(),
            default_context_ids.to_vec(),
            verification.confidence.unwrap_or(input.source.confidence),
            Some("verification_cells"),
        )?);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lifted_cell(
    input: &TestGapInputDocument,
    space_id: &Id,
    id: Id,
    dimension: u32,
    cell_type: &str,
    label: String,
    context_ids: Vec<Id>,
    confidence: Confidence,
    source_local_id: Option<&str>,
) -> RuntimeResult<TestGapLiftedCell> {
    Ok(TestGapLiftedCell {
        id,
        space_id: space_id.clone(),
        dimension,
        cell_type: cell_type.to_owned(),
        label,
        context_ids,
        provenance: fact_provenance(input, confidence, source_local_id)?,
    })
}

pub(super) fn lifted_incidences(
    input: &TestGapInputDocument,
    space_id: &Id,
) -> RuntimeResult<Vec<TestGapLiftedIncidence>> {
    let mut incidences = Vec::new();
    append_file_incidences(input, space_id, &mut incidences)?;
    append_symbol_incidences(input, space_id, &mut incidences)?;
    append_test_incidences(input, space_id, &mut incidences)?;
    append_coverage_incidences(input, space_id, &mut incidences)?;
    append_input_edge_incidences(input, space_id, &mut incidences)?;
    append_higher_order_incidences(input, space_id, &mut incidences)?;
    append_morphism_incidences(input, space_id, &mut incidences)?;
    append_law_incidences(input, space_id, &mut incidences)?;
    append_verification_incidences(input, space_id, &mut incidences)?;
    Ok(incidences)
}

fn append_file_incidences(
    input: &TestGapInputDocument,
    space_id: &Id,
    incidences: &mut Vec<TestGapLiftedIncidence>,
) -> RuntimeResult<()> {
    for file in &input.changed_files {
        for symbol_id in &file.symbol_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("contains-symbol", &file.id, symbol_id)?,
                file.id.clone(),
                symbol_id.clone(),
                "contains_symbol",
                input.source.confidence,
            )?);
        }
    }
    Ok(())
}

fn append_symbol_incidences(
    input: &TestGapInputDocument,
    space_id: &Id,
    incidences: &mut Vec<TestGapLiftedIncidence>,
) -> RuntimeResult<()> {
    for symbol in &input.symbols {
        for branch_id in &symbol.branch_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("has-branch", &symbol.id, branch_id)?,
                symbol.id.clone(),
                branch_id.clone(),
                "has_branch",
                input.source.confidence,
            )?);
        }
        for requirement_id in &symbol.requirement_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("implements-requirement", &symbol.id, requirement_id)?,
                symbol.id.clone(),
                requirement_id.clone(),
                "implements_requirement",
                input.source.confidence,
            )?);
        }
    }
    Ok(())
}

fn append_test_incidences(
    input: &TestGapInputDocument,
    space_id: &Id,
    incidences: &mut Vec<TestGapLiftedIncidence>,
) -> RuntimeResult<()> {
    for test in &input.tests {
        for target_id in &test.target_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("covered-by-test", target_id, &test.id)?,
                target_id.clone(),
                test.id.clone(),
                "covered_by_test",
                input.source.confidence,
            )?);
        }
        for branch_id in &test.branch_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("exercises-condition", &test.id, branch_id)?,
                test.id.clone(),
                branch_id.clone(),
                "exercises_condition",
                input.source.confidence,
            )?);
        }
        for requirement_id in &test.requirement_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("verifies-requirement", &test.id, requirement_id)?,
                test.id.clone(),
                requirement_id.clone(),
                "verifies_requirement",
                input.source.confidence,
            )?);
        }
    }
    Ok(())
}

fn append_coverage_incidences(
    input: &TestGapInputDocument,
    space_id: &Id,
    incidences: &mut Vec<TestGapLiftedIncidence>,
) -> RuntimeResult<()> {
    for coverage in &input.coverage {
        incidences.push(lifted_incidence(
            input,
            space_id,
            incidence_id("coverage-supports", &coverage.id, &coverage.target_id)?,
            coverage.id.clone(),
            coverage.target_id.clone(),
            "supports",
            coverage.confidence.unwrap_or(input.source.confidence),
        )?);
    }
    Ok(())
}

fn append_input_edge_incidences(
    input: &TestGapInputDocument,
    space_id: &Id,
    incidences: &mut Vec<TestGapLiftedIncidence>,
) -> RuntimeResult<()> {
    for edge in &input.dependency_edges {
        incidences.push(TestGapLiftedIncidence {
            id: edge.id.clone(),
            space_id: space_id.clone(),
            from_cell_id: edge.from_id.clone(),
            to_cell_id: edge.to_id.clone(),
            relation_type: serde_plain_dependency_relation_type(edge.relation_type),
            orientation: edge.orientation.unwrap_or(IncidenceOrientation::Directed),
            weight: None,
            provenance: fact_provenance(
                input,
                edge.confidence.unwrap_or(input.source.confidence),
                Some("dependency_edges"),
            )?,
        });
    }
    Ok(())
}

fn append_higher_order_incidences(
    input: &TestGapInputDocument,
    space_id: &Id,
    incidences: &mut Vec<TestGapLiftedIncidence>,
) -> RuntimeResult<()> {
    for incidence in &input.higher_order_incidences {
        incidences.push(TestGapLiftedIncidence {
            id: incidence.id.clone(),
            space_id: space_id.clone(),
            from_cell_id: incidence.from_id.clone(),
            to_cell_id: incidence.to_id.clone(),
            relation_type: incidence.relation_type.clone(),
            orientation: incidence
                .orientation
                .unwrap_or(IncidenceOrientation::Directed),
            weight: None,
            provenance: fact_provenance(
                input,
                incidence.confidence.unwrap_or(input.source.confidence),
                Some("higher_order_incidences"),
            )?,
        });
    }
    Ok(())
}

fn append_morphism_incidences(
    input: &TestGapInputDocument,
    space_id: &Id,
    incidences: &mut Vec<TestGapLiftedIncidence>,
) -> RuntimeResult<()> {
    for morphism in &input.morphisms {
        for source_id in &morphism.source_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("morphism-source", &morphism.id, source_id)?,
                morphism.id.clone(),
                source_id.clone(),
                "morphism_source",
                morphism.confidence.unwrap_or(input.source.confidence),
            )?);
        }
        for target_id in &morphism.target_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("morphism-target", &morphism.id, target_id)?,
                morphism.id.clone(),
                target_id.clone(),
                "morphism_target",
                morphism.confidence.unwrap_or(input.source.confidence),
            )?);
        }
        for law_id in &morphism.law_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("morphism-preserves-law", &morphism.id, law_id)?,
                morphism.id.clone(),
                law_id.clone(),
                "preserves_law",
                morphism.confidence.unwrap_or(input.source.confidence),
            )?);
        }
    }
    Ok(())
}

fn append_law_incidences(
    input: &TestGapInputDocument,
    space_id: &Id,
    incidences: &mut Vec<TestGapLiftedIncidence>,
) -> RuntimeResult<()> {
    for law in &input.laws {
        for applies_to_id in &law.applies_to_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("law-applies-to", &law.id, applies_to_id)?,
                law.id.clone(),
                applies_to_id.clone(),
                "applies_to",
                law.confidence.unwrap_or(input.source.confidence),
            )?);
        }
    }
    Ok(())
}

fn append_verification_incidences(
    input: &TestGapInputDocument,
    space_id: &Id,
    incidences: &mut Vec<TestGapLiftedIncidence>,
) -> RuntimeResult<()> {
    for verification in &input.verification_cells {
        for law_id in &verification.law_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("verification-closes-law", &verification.id, law_id)?,
                verification.id.clone(),
                law_id.clone(),
                "verifies_law",
                verification.confidence.unwrap_or(input.source.confidence),
            )?);
        }
        for morphism_id in &verification.morphism_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id(
                    "verification-closes-morphism",
                    &verification.id,
                    morphism_id,
                )?,
                verification.id.clone(),
                morphism_id.clone(),
                "verifies_morphism",
                verification.confidence.unwrap_or(input.source.confidence),
            )?);
        }
    }
    Ok(())
}

pub(super) fn lifted_incidence(
    input: &TestGapInputDocument,
    space_id: &Id,
    id: Id,
    from_cell_id: Id,
    to_cell_id: Id,
    relation_type: &str,
    confidence: Confidence,
) -> RuntimeResult<TestGapLiftedIncidence> {
    Ok(TestGapLiftedIncidence {
        id,
        space_id: space_id.clone(),
        from_cell_id,
        to_cell_id,
        relation_type: relation_type.to_owned(),
        orientation: IncidenceOrientation::Directed,
        weight: None,
        provenance: fact_provenance(input, confidence, Some("lifted_incidences"))?,
    })
}
