{
        self.match_index += 1;
        let match_id = match id(format!(
            "semantic:rust:match:{}:{}:{}:{}",
            slug(&self.change.path),
            self.revision.as_str(),
            self.parent_slug,
            self.match_index
        )) {
            Ok(value) => value,
            Err(_) => return,
        };
        if push_higher_order_cell(
            &mut self.model,
            match_id.clone(),
            "rust_match",
            format!("Rust match expression {}", self.match_index),
            0,
            self.change,
            self.diff_evidence_id,
            0.68,
        )
        .is_err()
        {
            return;
        }
        self.semantic_cells.push(SemanticCell {
            id: match_id.clone(),
            key: format!(
                "rust:match:{}:{}:{}",
                slug(&self.change.path),
                self.parent_slug,
                self.match_index
            ),
            cell_type: "rust_match".to_owned(),
        });
        let _ = push_higher_order_incidence(
            &mut self.model,
            format!(
                "incidence:semantic:rust:function-contains-match:{}:{}:{}:{}",
                slug(&self.change.path),
                self.revision.as_str(),
                self.parent_slug,
                self.match_index
            ),
            self.parent_id.clone(),
            match_id.clone(),
            "contains_match",
            self.diff_evidence_id,
            0.68,
        );
        for (arm_index, _) in node.arms.iter().enumerate() {
            let arm_number = arm_index + 1;
            let Ok(arm_id) = id(format!(
                "semantic:rust:match-arm:{}:{}:{}:{}:{}",
                slug(&self.change.path),
                self.revision.as_str(),
                self.parent_slug,
                self.match_index,
                arm_number
            )) else {
                continue;
            };
            let _ = push_higher_order_cell(
                &mut self.model,
                arm_id.clone(),
                "rust_match_arm",
                format!("Rust match arm {arm_number}"),
                0,
                self.change,
                self.diff_evidence_id,
                0.66,
            );
            self.semantic_cells.push(SemanticCell {
                id: arm_id.clone(),
                key: format!(
                    "rust:match-arm:{}:{}:{}:{}",
                    slug(&self.change.path),
                    self.parent_slug,
                    self.match_index,
                    arm_number
                ),
                cell_type: "rust_match_arm".to_owned(),
            });
            let _ = push_higher_order_incidence(
                &mut self.model,
                format!(
                    "incidence:semantic:rust:match-contains-arm:{}:{}:{}:{}:{}",
                    slug(&self.change.path),
                    self.revision.as_str(),
                    self.parent_slug,
                    self.match_index,
                    arm_number
                ),
                match_id.clone(),
                arm_id,
                "contains_match_arm",
                self.diff_evidence_id,
                0.66,
            );
        }
        syn::visit::visit_expr_match(self, node);
}
