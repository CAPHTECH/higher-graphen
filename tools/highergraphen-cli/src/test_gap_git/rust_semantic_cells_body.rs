{
    let Ok(file) = syn::parse_file(contents) else {
        return Ok(Vec::new());
    };
    let mut semantic_cells = Vec::new();
    let file_cell_id = id(format!(
        "semantic:rust:file:{}:{}",
        slug(&change.path),
        revision.as_str()
    ))?;
    push_higher_order_cell(
        model,
        file_cell_id.clone(),
        "rust_file_revision",
        format!(
            "Rust semantic file {} at {} from {}",
            change.path,
            revision.as_str(),
            source_path
        ),
        0,
        change,
        diff_evidence_id,
        0.7,
    )?;
    semantic_cells.push(SemanticCell {
        id: file_cell_id.clone(),
        key: format!("rust:file:{}", slug(&change.path)),
        cell_type: "rust_file_revision".to_owned(),
    });

    for item in &file.items {
        match item {
            syn::Item::Fn(item_fn) => {
                let function_id = id(format!(
                    "semantic:rust:function:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    slug(&item_fn.sig.ident.to_string())
                ))?;
                push_higher_order_cell(
                    model,
                    function_id.clone(),
                    "rust_function",
                    format!("Rust function {}", item_fn.sig.ident),
                    0,
                    change,
                    diff_evidence_id,
                    0.72,
                )?;
                semantic_cells.push(SemanticCell {
                    id: function_id.clone(),
                    key: format!(
                        "rust:function:{}:{}",
                        slug(&change.path),
                        slug(&item_fn.sig.ident.to_string())
                    ),
                    cell_type: "rust_function".to_owned(),
                });
                push_higher_order_incidence(
                    model,
                    format!(
                        "incidence:semantic:rust:file-contains-function:{}:{}:{}",
                        slug(&change.path),
                        revision.as_str(),
                        slug(&item_fn.sig.ident.to_string())
                    ),
                    file_cell_id.clone(),
                    function_id.clone(),
                    "contains_function",
                    diff_evidence_id,
                    0.72,
                )?;
                let mut visitor = RustSemanticVisitor::new(
                    change,
                    revision,
                    &function_id,
                    &slug(&item_fn.sig.ident.to_string()),
                    diff_evidence_id,
                );
                visitor.visit_block(&item_fn.block);
                semantic_cells.extend(visitor.semantic_cells);
                model.extend(visitor.model);
            }
            syn::Item::Struct(item_struct) => {
                let struct_id = id(format!(
                    "semantic:rust:struct:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    slug(&item_struct.ident.to_string())
                ))?;
                push_higher_order_cell(
                    model,
                    struct_id.clone(),
                    "rust_struct",
                    format!("Rust struct {}", item_struct.ident),
                    0,
                    change,
                    diff_evidence_id,
                    0.72,
                )?;
                semantic_cells.push(SemanticCell {
                    id: struct_id.clone(),
                    key: format!(
                        "rust:struct:{}:{}",
                        slug(&change.path),
                        slug(&item_struct.ident.to_string())
                    ),
                    cell_type: "rust_struct".to_owned(),
                });
                push_higher_order_incidence(
                    model,
                    format!(
                        "incidence:semantic:rust:file-contains-struct:{}:{}:{}",
                        slug(&change.path),
                        revision.as_str(),
                        slug(&item_struct.ident.to_string())
                    ),
                    file_cell_id.clone(),
                    struct_id,
                    "contains_struct",
                    diff_evidence_id,
                    0.72,
                )?;
            }
            syn::Item::Enum(item_enum) => {
                let enum_id = id(format!(
                    "semantic:rust:enum:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    slug(&item_enum.ident.to_string())
                ))?;
                push_higher_order_cell(
                    model,
                    enum_id.clone(),
                    "rust_enum",
                    format!("Rust enum {}", item_enum.ident),
                    0,
                    change,
                    diff_evidence_id,
                    0.72,
                )?;
                semantic_cells.push(SemanticCell {
                    id: enum_id.clone(),
                    key: format!(
                        "rust:enum:{}:{}",
                        slug(&change.path),
                        slug(&item_enum.ident.to_string())
                    ),
                    cell_type: "rust_enum".to_owned(),
                });
                push_higher_order_incidence(
                    model,
                    format!(
                        "incidence:semantic:rust:file-contains-enum:{}:{}:{}",
                        slug(&change.path),
                        revision.as_str(),
                        slug(&item_enum.ident.to_string())
                    ),
                    file_cell_id.clone(),
                    enum_id.clone(),
                    "contains_enum",
                    diff_evidence_id,
                    0.72,
                )?;
                for variant in &item_enum.variants {
                    let variant_id = id(format!(
                        "semantic:rust:enum-variant:{}:{}:{}:{}",
                        slug(&change.path),
                        revision.as_str(),
                        slug(&item_enum.ident.to_string()),
                        slug(&variant.ident.to_string())
                    ))?;
                    push_higher_order_cell(
                        model,
                        variant_id.clone(),
                        "rust_enum_variant",
                        format!("Rust enum variant {}::{}", item_enum.ident, variant.ident),
                        0,
                        change,
                        diff_evidence_id,
                        0.7,
                    )?;
                    semantic_cells.push(SemanticCell {
                        id: variant_id.clone(),
                        key: format!(
                            "rust:enum-variant:{}:{}:{}",
                            slug(&change.path),
                            slug(&item_enum.ident.to_string()),
                            slug(&variant.ident.to_string())
                        ),
                        cell_type: "rust_enum_variant".to_owned(),
                    });
                    push_higher_order_incidence(
                        model,
                        format!(
                            "incidence:semantic:rust:enum-contains-variant:{}:{}:{}:{}",
                            slug(&change.path),
                            revision.as_str(),
                            slug(&item_enum.ident.to_string()),
                            slug(&variant.ident.to_string())
                        ),
                        enum_id.clone(),
                        variant_id,
                        "contains_variant",
                        diff_evidence_id,
                        0.7,
                    )?;
                }
            }
            syn::Item::Impl(item_impl) => {
                let impl_index = semantic_cells
                    .iter()
                    .filter(|cell| cell.cell_type == "rust_impl")
                    .count()
                    + 1;
                let impl_id = id(format!(
                    "semantic:rust:impl:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    impl_index
                ))?;
                push_higher_order_cell(
                    model,
                    impl_id.clone(),
                    "rust_impl",
                    format!("Rust impl block {impl_index}"),
                    0,
                    change,
                    diff_evidence_id,
                    0.68,
                )?;
                semantic_cells.push(SemanticCell {
                    id: impl_id.clone(),
                    key: format!("rust:impl:{}:{}", slug(&change.path), impl_index),
                    cell_type: "rust_impl".to_owned(),
                });
                push_higher_order_incidence(
                    model,
                    format!(
                        "incidence:semantic:rust:file-contains-impl:{}:{}:{}",
                        slug(&change.path),
                        revision.as_str(),
                        impl_index
                    ),
                    file_cell_id.clone(),
                    impl_id.clone(),
                    "contains_impl",
                    diff_evidence_id,
                    0.68,
                )?;
                for impl_item in &item_impl.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        let method_id = id(format!(
                            "semantic:rust:method:{}:{}:{}:{}",
                            slug(&change.path),
                            revision.as_str(),
                            impl_index,
                            slug(&method.sig.ident.to_string())
                        ))?;
                        push_higher_order_cell(
                            model,
                            method_id.clone(),
                            "rust_method",
                            format!("Rust method {}", method.sig.ident),
                            0,
                            change,
                            diff_evidence_id,
                            0.7,
                        )?;
                        semantic_cells.push(SemanticCell {
                            id: method_id.clone(),
                            key: format!(
                                "rust:method:{}:{}:{}",
                                slug(&change.path),
                                impl_index,
                                slug(&method.sig.ident.to_string())
                            ),
                            cell_type: "rust_method".to_owned(),
                        });
                        push_higher_order_incidence(
                            model,
                            format!(
                                "incidence:semantic:rust:impl-contains-method:{}:{}:{}:{}",
                                slug(&change.path),
                                revision.as_str(),
                                impl_index,
                                slug(&method.sig.ident.to_string())
                            ),
                            impl_id.clone(),
                            method_id.clone(),
                            "contains_method",
                            diff_evidence_id,
                            0.7,
                        )?;
                        let mut visitor = RustSemanticVisitor::new(
                            change,
                            revision,
                            &method_id,
                            &format!("impl-{impl_index}-{}", slug(&method.sig.ident.to_string())),
                            diff_evidence_id,
                        );
                        visitor.visit_block(&method.block);
                        semantic_cells.extend(visitor.semantic_cells);
                        model.extend(visitor.model);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(semantic_cells)
}
