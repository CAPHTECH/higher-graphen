use super::*;
use serde_json::json;

fn id(value: &str) -> Id {
    Id::new(value).expect("valid id")
}

fn base_context() -> Context {
    Context::new(id("ctx:base"), "Base")
        .expect("valid context")
        .with_elements([id("a"), id("b"), id("c")])
}

fn left_context() -> Context {
    Context::new(id("ctx:left"), "Left")
        .expect("valid context")
        .with_elements([id("a"), id("b")])
}

fn right_context() -> Context {
    Context::new(id("ctx:right"), "Right")
        .expect("valid context")
        .with_elements([id("b"), id("c")])
}

fn cover() -> Cover {
    Cover::new(
        id("cover:base"),
        id("ctx:base"),
        [id("ctx:left"), id("ctx:right")],
    )
}

fn compatible_check() -> GluingCheck<i32> {
    GluingCheck::new(id("glue:check"), cover(), id("section:global"))
        .with_contexts([base_context(), left_context(), right_context()])
        .with_local_sections([
            Section::new(id("section:left"), id("ctx:left"))
                .with_assignment(id("a"), 1)
                .with_assignment(id("b"), 2),
            Section::new(id("section:right"), id("ctx:right"))
                .with_assignment(id("b"), 2)
                .with_assignment(id("c"), 3),
        ])
}

#[test]
fn compatible_local_sections_glue_into_global_section() {
    let result = compatible_check().evaluate();

    let GluingCheckResult::Gluable {
        glued_section,
        checked_overlap_count,
        ..
    } = result
    else {
        panic!("expected compatible sections to glue");
    };

    assert_eq!(checked_overlap_count, 1);
    assert_eq!(glued_section.context_id, id("ctx:base"));
    assert_eq!(glued_section.assignments.get(&id("a")), Some(&1));
    assert_eq!(glued_section.assignments.get(&id("b")), Some(&2));
    assert_eq!(glued_section.assignments.get(&id("c")), Some(&3));
}

#[test]
fn incompatible_local_assignments_report_overlap_witness() {
    let result = GluingCheck::new(id("glue:check"), cover(), id("section:global"))
        .with_contexts([base_context(), left_context(), right_context()])
        .with_local_sections([
            Section::new(id("section:left"), id("ctx:left"))
                .with_assignment(id("a"), 1)
                .with_assignment(id("b"), 2),
            Section::new(id("section:right"), id("ctx:right"))
                .with_assignment(id("b"), 9)
                .with_assignment(id("c"), 3),
        ])
        .evaluate();

    let GluingCheckResult::NotGluable {
        obstructions,
        checked_overlap_count,
        ..
    } = result
    else {
        panic!("expected incompatible sections to be rejected");
    };

    assert_eq!(checked_overlap_count, 1);
    assert!(obstructions.iter().any(|obstruction| matches!(
        obstruction,
        GluingObstruction::IncompatibleOverlap { witness }
            if witness.element_id == id("b")
                && witness.left_value == 2
                && witness.right_value == 9
    )));
}

#[test]
fn missing_cover_member_reports_not_gluable() {
    let result = GluingCheck::new(id("glue:check"), cover(), id("section:global"))
        .with_contexts([base_context(), left_context(), right_context()])
        .with_local_section(
            Section::new(id("section:left"), id("ctx:left"))
                .with_assignment(id("a"), 1)
                .with_assignment(id("b"), 2),
        )
        .evaluate();

    let GluingCheckResult::NotGluable { obstructions, .. } = result else {
        panic!("expected missing local cover member to be rejected");
    };

    assert!(obstructions.iter().any(|obstruction| matches!(
        obstruction,
        GluingObstruction::MissingCoverMember { context_id }
            if context_id == &id("ctx:right")
    )));
}

#[test]
fn restriction_applies_selected_assignments_and_rejects_wrong_source() {
    let restriction = Restriction::new(
        id("restriction:left-to-overlap"),
        id("ctx:left"),
        id("ctx:overlap"),
        [id("b")],
    );
    let left_section = Section::new(id("section:left"), id("ctx:left"))
        .with_assignment(id("a"), 1)
        .with_assignment(id("b"), 2);

    let restricted = restriction
        .apply(id("section:left-overlap"), &left_section)
        .expect("restriction applies to source section");

    assert_eq!(restricted.context_id, id("ctx:overlap"));
    assert_eq!(restricted.assignments.len(), 1);
    assert_eq!(restricted.assignments.get(&id("b")), Some(&2));

    let wrong_section: Section<i32> = Section::new(id("section:right"), id("ctx:right"));
    let error = restriction
        .apply(id("section:wrong"), &wrong_section)
        .expect_err("wrong source context rejected");
    assert_eq!(error.code(), "malformed_field");
}

#[test]
fn serde_rejects_malformed_context_records() {
    let unknown_field = json!({
        "id": "ctx:base",
        "name": "Base",
        "element_ids": [],
        "unexpected": true
    });
    assert!(serde_json::from_value::<Context>(unknown_field).is_err());

    let empty_id = json!({
        "id": " ",
        "name": "Base",
        "element_ids": []
    });
    assert!(serde_json::from_value::<Context>(empty_id).is_err());
}
