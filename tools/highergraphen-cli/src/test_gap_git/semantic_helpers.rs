fn semantic_expr_mentions_error_path(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| matches!(segment.ident.to_string().as_str(), "panic"))
            .unwrap_or(false),
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn push_higher_order_cell(
    model: &mut StructuralModel,
    cell_id: Id,
    cell_type: &str,
    label: String,
    dimension: u32,
    change: &GitChange,
    diff_evidence_id: &Id,
    confidence_value: f64,
) -> Result<(), String> {
    if model
        .higher_order_cells
        .iter()
        .any(|cell| cell.id == cell_id)
    {
        return Ok(());
    }
    model.higher_order_cells.push(TestGapHigherOrderCell {
        id: cell_id,
        cell_type: cell_type.to_owned(),
        label,
        dimension,
        context_ids: test_gap_context_ids_for_path(&change.path)?,
        source_ids: vec![diff_evidence_id.clone()],
        confidence: Some(confidence(confidence_value)?),
    });
    Ok(())
}

fn push_higher_order_incidence(
    model: &mut StructuralModel,
    incidence_id: String,
    from_id: Id,
    to_id: Id,
    relation_type: &str,
    diff_evidence_id: &Id,
    confidence_value: f64,
) -> Result<(), String> {
    let incidence_id = id(incidence_id)?;
    if model
        .higher_order_incidences
        .iter()
        .any(|incidence| incidence.id == incidence_id)
    {
        return Ok(());
    }
    model
        .higher_order_incidences
        .push(TestGapHigherOrderIncidence {
            id: incidence_id,
            from_id,
            to_id,
            relation_type: relation_type.to_owned(),
            orientation: None,
            source_ids: vec![diff_evidence_id.clone()],
            confidence: Some(confidence(confidence_value)?),
        });
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn push_structural_symbol(
    model: &mut StructuralModel,
    changes: &[GitChange],
    diff_evidence_id: &Id,
    file_path: &str,
    symbol_id: &str,
    name: &str,
    path: &str,
    kind: TestGapSymbolKind,
) -> Result<(), String> {
    if !changes.iter().any(|change| change.path == file_path) {
        return Ok(());
    }
    model.symbols.push(TestGapInputSymbol {
        id: id(symbol_id)?,
        file_id: file_id(file_path)?,
        name: name.to_owned(),
        kind,
        visibility: TestGapVisibility::Public,
        public_api: true,
        path: Some(path.to_owned()),
        line_start: None,
        line_end: None,
        branch_ids: Vec::new(),
        requirement_ids: Vec::new(),
        context_ids: test_gap_context_ids_for_path(file_path)?,
        source_ids: vec![diff_evidence_id.clone()],
    });
    model.higher_order_cells.push(TestGapHigherOrderCell {
        id: id(symbol_id)?,
        cell_type: symbol_id
            .split(':')
            .next()
            .unwrap_or("structure")
            .to_owned(),
        label: name.to_owned(),
        dimension: 0,
        context_ids: test_gap_context_ids_for_path(file_path)?,
        source_ids: vec![diff_evidence_id.clone()],
        confidence: Some(confidence(0.8)?),
    });
    Ok(())
}

fn push_structural_symbol_unconditional(
    model: &mut StructuralModel,
    diff_evidence_id: &Id,
    file_path: &str,
    symbol_id: &str,
    name: &str,
    path: &str,
    kind: TestGapSymbolKind,
) -> Result<(), String> {
    model.symbols.push(TestGapInputSymbol {
        id: id(symbol_id)?,
        file_id: file_id(file_path)?,
        name: name.to_owned(),
        kind,
        visibility: TestGapVisibility::Public,
        public_api: true,
        path: Some(path.to_owned()),
        line_start: None,
        line_end: None,
        branch_ids: Vec::new(),
        requirement_ids: Vec::new(),
        context_ids: test_gap_context_ids_for_path(file_path)?,
        source_ids: vec![diff_evidence_id.clone()],
    });
    model.higher_order_cells.push(TestGapHigherOrderCell {
        id: id(symbol_id)?,
        cell_type: symbol_id
            .split(':')
            .next()
            .unwrap_or("structure")
            .to_owned(),
        label: name.to_owned(),
        dimension: 0,
        context_ids: test_gap_context_ids_for_path(file_path)?,
        source_ids: vec![diff_evidence_id.clone()],
        confidence: Some(confidence(0.8)?),
    });
    Ok(())
}

fn push_law_symbol(
    model: &mut StructuralModel,
    changes: &[GitChange],
    diff_evidence_id: &Id,
    file_paths: &[&str],
    symbol_id: &str,
    name: &str,
) -> Result<(), String> {
    let Some(file_path) = file_paths
        .iter()
        .find(|file_path| changes.iter().any(|change| change.path == **file_path))
    else {
        return Ok(());
    };
    model.symbols.push(TestGapInputSymbol {
        id: id(symbol_id)?,
        file_id: file_id(file_path)?,
        name: name.to_owned(),
        kind: TestGapSymbolKind::Unknown,
        visibility: TestGapVisibility::Public,
        public_api: true,
        path: Some(symbol_id.to_owned()),
        line_start: None,
        line_end: None,
        branch_ids: Vec::new(),
        requirement_ids: Vec::new(),
        context_ids: test_gap_context_ids_for_path(file_path)?,
        source_ids: vec![diff_evidence_id.clone()],
    });
    model.laws.push(TestGapInputLaw {
        id: id(symbol_id)?,
        summary: name.to_owned(),
        applies_to_ids: Vec::new(),
        requirement_ids: Vec::new(),
        source_ids: vec![diff_evidence_id.clone()],
        expected_verification: Some("policy_accepted_verification".to_owned()),
        confidence: Some(confidence(0.82)?),
    });
    Ok(())
}

fn push_structural_edge(
    model: &mut StructuralModel,
    edge_id: &str,
    from_id: &str,
    to_id: &str,
    relation_type: TestGapDependencyRelationType,
    diff_evidence_id: &Id,
) -> Result<(), String> {
    let from_id = id(from_id)?;
    let to_id = id(to_id)?;
    if !has_structural_symbol(&model.symbols, &from_id)
        || !has_structural_symbol(&model.symbols, &to_id)
    {
        return Ok(());
    }
    model.dependency_edges.push(TestGapInputDependencyEdge {
        id: id(edge_id)?,
        from_id: from_id.clone(),
        to_id: to_id.clone(),
        relation_type,
        orientation: None,
        source_ids: vec![diff_evidence_id.clone()],
        confidence: Some(confidence(0.78)?),
    });
    model
        .higher_order_incidences
        .push(TestGapHigherOrderIncidence {
            id: id(format!("incidence:{edge_id}"))?,
            from_id,
            to_id,
            relation_type: serde_plain_dependency_relation_type(relation_type),
            orientation: None,
            source_ids: vec![diff_evidence_id.clone()],
            confidence: Some(confidence(0.78)?),
        });
    Ok(())
}

fn push_higher_order_morphism(
    model: &mut StructuralModel,
    morphism_id: &str,
    morphism_type: &str,
    source_ids: &[&str],
    target_ids: &[&str],
    law_ids: &[&str],
    _diff_evidence_id: &Id,
) -> Result<(), String> {
    let source_ids = ids_present_in_model(model, source_ids)?;
    let target_ids = ids_present_in_model(model, target_ids)?;
    if source_ids.is_empty() || target_ids.is_empty() {
        return Ok(());
    }
    let law_ids = ids_present_in_model(model, law_ids)?;
    let morphism_id = id(morphism_id)?;
    model.morphisms.push(TestGapInputMorphism {
        id: morphism_id.clone(),
        morphism_type: morphism_type.to_owned(),
        source_ids,
        target_ids,
        law_ids: law_ids.clone(),
        requirement_ids: Vec::new(),
        expected_verification: Some("policy_accepted_verification".to_owned()),
        confidence: Some(confidence(0.8)?),
    });
    for law_id in law_ids {
        if let Some(law) = model.laws.iter_mut().find(|law| law.id == law_id) {
            push_unique_id(&mut law.applies_to_ids, morphism_id.clone());
        }
    }
    Ok(())
}

fn ids_present_in_model(model: &StructuralModel, ids: &[&str]) -> Result<Vec<Id>, String> {
    ids.iter()
        .filter(|value| {
            model
                .higher_order_cells
                .iter()
                .any(|cell| cell.id.as_str() == **value)
                || model.laws.iter().any(|law| law.id.as_str() == **value)
        })
        .map(|value| id(*value))
        .collect()
}

fn serde_plain_dependency_relation_type(relation_type: TestGapDependencyRelationType) -> String {
    match relation_type {
        TestGapDependencyRelationType::Contains => "contains",
        TestGapDependencyRelationType::ImplementsRequirement => "implements_requirement",
        TestGapDependencyRelationType::HasBranch => "has_branch",
        TestGapDependencyRelationType::CoveredByTest => "covered_by_test",
        TestGapDependencyRelationType::ExercisesCondition => "exercises_condition",
        TestGapDependencyRelationType::DependsOn => "depends_on",
        TestGapDependencyRelationType::Supports => "supports",
        TestGapDependencyRelationType::InContext => "in_context",
        TestGapDependencyRelationType::Custom => "custom",
    }
    .to_owned()
}

fn has_structural_symbol(symbols: &[TestGapInputSymbol], symbol_id: &Id) -> bool {
    symbols.iter().any(|symbol| &symbol.id == symbol_id)
}
