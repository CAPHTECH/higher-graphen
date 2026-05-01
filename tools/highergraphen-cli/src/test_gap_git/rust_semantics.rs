fn push_rust_semantic_cells(
    model: &mut StructuralModel,
    change: &GitChange,
    revision: SemanticRevision,
    source_path: &str,
    contents: &str,
    diff_evidence_id: &Id,
) -> Result<Vec<SemanticCell>, String> {
    include!("rust_semantic_cells_body.rs")
}

fn push_json_schema_semantic_cells(
    model: &mut StructuralModel,
    change: &GitChange,
    revision: SemanticRevision,
    source_path: &str,
    contents: &str,
    diff_evidence_id: &Id,
) -> Result<Vec<SemanticCell>, String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(contents) else {
        return Ok(Vec::new());
    };
    let schema_id = id(format!(
        "semantic:json-schema:document:{}:{}",
        slug(&change.path),
        revision.as_str()
    ))?;
    push_higher_order_cell(
        model,
        schema_id.clone(),
        "json_schema_revision",
        format!(
            "JSON schema {} at {} from {}",
            change.path,
            revision.as_str(),
            source_path
        ),
        0,
        change,
        diff_evidence_id,
        0.74,
    )?;
    let mut semantic_cells = vec![SemanticCell {
        id: schema_id.clone(),
        key: format!("json-schema:document:{}", slug(&change.path)),
        cell_type: "json_schema_revision".to_owned(),
    }];
    push_json_schema_properties(
        model,
        change,
        revision,
        &schema_id,
        "#".to_owned(),
        &value,
        diff_evidence_id,
        &mut semantic_cells,
    )?;
    Ok(semantic_cells)
}

#[allow(clippy::too_many_arguments)]
fn push_json_schema_properties(
    model: &mut StructuralModel,
    change: &GitChange,
    revision: SemanticRevision,
    schema_id: &Id,
    pointer: String,
    value: &serde_json::Value,
    diff_evidence_id: &Id,
    semantic_cells: &mut Vec<SemanticCell>,
) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    if let Some(properties) = object.get("properties").and_then(|value| value.as_object()) {
        for property_name in properties.keys() {
            let property_id = id(format!(
                "semantic:json-schema:property:{}:{}:{}:{}",
                slug(&change.path),
                revision.as_str(),
                slug(&pointer),
                slug(property_name)
            ))?;
            push_higher_order_cell(
                model,
                property_id.clone(),
                "json_schema_property",
                format!("JSON schema property {pointer}/{property_name}"),
                0,
                change,
                diff_evidence_id,
                0.74,
            )?;
            semantic_cells.push(SemanticCell {
                id: property_id.clone(),
                key: format!(
                    "json-schema:property:{}:{}:{}",
                    slug(&change.path),
                    slug(&pointer),
                    slug(property_name)
                ),
                cell_type: "json_schema_property".to_owned(),
            });
            push_higher_order_incidence(
                model,
                format!(
                    "incidence:semantic:json-schema:declares-property:{}:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    slug(&pointer),
                    slug(property_name)
                ),
                schema_id.clone(),
                property_id,
                "declares_property",
                diff_evidence_id,
                0.74,
            )?;
        }
    }
    if let Some(defs) = object.get("$defs").and_then(|value| value.as_object()) {
        for (def_name, def_value) in defs {
            push_json_schema_properties(
                model,
                change,
                revision,
                schema_id,
                format!("{pointer}/$defs/{def_name}"),
                def_value,
                diff_evidence_id,
                semantic_cells,
            )?;
        }
    }
    Ok(())
}

fn push_semantic_delta_structure(
    model: &mut StructuralModel,
    change: &GitChange,
    base_cells: &[SemanticCell],
    head_cells: &[SemanticCell],
    diff_evidence_id: &Id,
) -> Result<(), String> {
    if base_cells.is_empty() && head_cells.is_empty() {
        return Ok(());
    }
    push_semantic_law(
        model,
        "law:test-gap:semantic-delta-is-explicit",
        "base/head semantic cells expose preservation, addition, and deletion morphisms",
        diff_evidence_id,
    )?;
    push_semantic_law(
        model,
        "law:test-gap:semantic-delta-has-verification",
        "changed semantic delta morphisms require accepted verification cells",
        diff_evidence_id,
    )?;

    let base_by_key = base_cells
        .iter()
        .map(|cell| (cell.key.as_str(), cell))
        .collect::<BTreeMap<_, _>>();
    let head_by_key = head_cells
        .iter()
        .map(|cell| (cell.key.as_str(), cell))
        .collect::<BTreeMap<_, _>>();
    for (key, base_cell) in &base_by_key {
        if let Some(head_cell) = head_by_key.get(key) {
            push_semantic_morphism(
                model,
                change,
                "semantic_preservation",
                base_cell,
                Some(head_cell),
                &[
                    "law:test-gap:semantic-delta-is-explicit",
                    "law:test-gap:semantic-delta-has-verification",
                ],
                diff_evidence_id,
            )?;
        } else {
            push_semantic_morphism(
                model,
                change,
                "semantic_deletion",
                base_cell,
                None,
                &[
                    "law:test-gap:semantic-delta-is-explicit",
                    "law:test-gap:semantic-delta-has-verification",
                ],
                diff_evidence_id,
            )?;
        }
    }
    for (key, head_cell) in &head_by_key {
        if !base_by_key.contains_key(key) {
            push_semantic_morphism(
                model,
                change,
                "semantic_addition",
                head_cell,
                None,
                &[
                    "law:test-gap:semantic-delta-is-explicit",
                    "law:test-gap:semantic-delta-has-verification",
                ],
                diff_evidence_id,
            )?;
        }
    }
    Ok(())
}

fn push_semantic_law(
    model: &mut StructuralModel,
    law_id: &str,
    summary: &str,
    diff_evidence_id: &Id,
) -> Result<(), String> {
    let law_id = id(law_id)?;
    if model.laws.iter().any(|law| law.id == law_id) {
        return Ok(());
    }
    model.laws.push(TestGapInputLaw {
        id: law_id,
        summary: summary.to_owned(),
        applies_to_ids: Vec::new(),
        requirement_ids: Vec::new(),
        source_ids: vec![diff_evidence_id.clone()],
        expected_verification: Some("policy_accepted_verification".to_owned()),
        confidence: Some(confidence(0.76)?),
    });
    Ok(())
}

fn push_semantic_morphism(
    model: &mut StructuralModel,
    change: &GitChange,
    morphism_type: &str,
    source_cell: &SemanticCell,
    target_cell: Option<&SemanticCell>,
    law_ids: &[&str],
    _diff_evidence_id: &Id,
) -> Result<(), String> {
    let morphism_id = id(format!(
        "morphism:test-gap:{}:{}:{}",
        morphism_type,
        slug(&change.path),
        slug(&source_cell.key)
    ))?;
    if model
        .morphisms
        .iter()
        .any(|morphism| morphism.id == morphism_id)
    {
        return Ok(());
    }
    let mut source_ids = vec![source_cell.id.clone()];
    let target_ids = if let Some(target_cell) = target_cell {
        vec![target_cell.id.clone()]
    } else if morphism_type == "semantic_addition" {
        source_ids = vec![file_id(&change.path)?];
        vec![source_cell.id.clone()]
    } else {
        vec![file_id(&change.path)?]
    };
    let law_ids = law_ids
        .iter()
        .filter_map(|law_id| id(*law_id).ok())
        .filter(|law_id| model.laws.iter().any(|law| &law.id == law_id))
        .collect::<Vec<_>>();
    model.morphisms.push(TestGapInputMorphism {
        id: morphism_id.clone(),
        morphism_type: morphism_type.to_owned(),
        source_ids,
        target_ids,
        law_ids: law_ids.clone(),
        requirement_ids: Vec::new(),
        expected_verification: semantic_delta_expected_verification(change),
        confidence: Some(confidence(0.7)?),
    });
    for law_id in law_ids {
        if let Some(law) = model.laws.iter_mut().find(|law| law.id == law_id) {
            push_unique_id(&mut law.applies_to_ids, morphism_id.clone());
        }
    }
    Ok(())
}

fn semantic_delta_expected_verification(change: &GitChange) -> Option<String> {
    if matches!(
        change.path.as_str(),
        "tools/highergraphen-cli/src/semantic_proof_artifact.rs"
            | "tools/highergraphen-cli/src/semantic_proof_backend.rs"
            | "tools/highergraphen-cli/src/semantic_proof_reinput.rs"
    ) {
        None
    } else {
        Some("policy_accepted_verification".to_owned())
    }
}

struct RustSemanticVisitor<'a> {
    change: &'a GitChange,
    revision: SemanticRevision,
    parent_id: &'a Id,
    parent_slug: String,
    diff_evidence_id: &'a Id,
    model: StructuralModel,
    semantic_cells: Vec<SemanticCell>,
    match_index: usize,
    error_index: usize,
}

impl<'a> RustSemanticVisitor<'a> {
    fn new(
        change: &'a GitChange,
        revision: SemanticRevision,
        parent_id: &'a Id,
        parent_slug: &str,
        diff_evidence_id: &'a Id,
    ) -> Self {
        Self {
            change,
            revision,
            parent_id,
            parent_slug: parent_slug.to_owned(),
            diff_evidence_id,
            model: StructuralModel::default(),
            semantic_cells: Vec::new(),
            match_index: 0,
            error_index: 0,
        }
    }
}

impl Visit<'_> for RustSemanticVisitor<'_> {
    fn visit_expr_match(&mut self, node: &syn::ExprMatch) {
    include!("visit_expr_match_body.rs")
}

    fn visit_expr_call(&mut self, node: &syn::ExprCall) {
        if semantic_expr_mentions_error_path(&node.func) {
            self.push_error_path_cell();
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_macro(&mut self, node: &syn::ExprMacro) {
        let macro_name = node
            .mac
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();
        if matches!(macro_name.as_str(), "panic" | "todo" | "unimplemented") {
            self.push_error_path_cell();
        }
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        if matches!(node.method.to_string().as_str(), "unwrap" | "expect") {
            self.push_error_path_cell();
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

impl RustSemanticVisitor<'_> {
    fn push_error_path_cell(&mut self) {
        self.error_index += 1;
        let Ok(error_id) = id(format!(
            "semantic:rust:error-path:{}:{}:{}:{}",
            slug(&self.change.path),
            self.revision.as_str(),
            self.parent_slug,
            self.error_index
        )) else {
            return;
        };
        let _ = push_higher_order_cell(
            &mut self.model,
            error_id.clone(),
            "rust_error_path",
            format!("Rust error path {}", self.error_index),
            0,
            self.change,
            self.diff_evidence_id,
            0.64,
        );
        self.semantic_cells.push(SemanticCell {
            id: error_id.clone(),
            key: format!(
                "rust:error-path:{}:{}:{}",
                slug(&self.change.path),
                self.parent_slug,
                self.error_index
            ),
            cell_type: "rust_error_path".to_owned(),
        });
        let _ = push_higher_order_incidence(
            &mut self.model,
            format!(
                "incidence:semantic:rust:function-contains-error-path:{}:{}:{}:{}",
                slug(&self.change.path),
                self.revision.as_str(),
                self.parent_slug,
                self.error_index
            ),
            self.parent_id.clone(),
            error_id,
            "contains_error_path",
            self.diff_evidence_id,
            0.64,
        );
    }
}
