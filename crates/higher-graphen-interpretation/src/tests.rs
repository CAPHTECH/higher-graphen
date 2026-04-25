use super::*;

struct Fixture {
    package: InterpretationPackage,
    component_mapping: TypeMapping,
    invariant: InvariantTemplate,
    projection: ProjectionTemplate,
    adapter: LiftAdapterDefinition,
}

fn id(value: &str) -> Id {
    Id::new(value).expect("test id should be valid")
}

fn component_mapping() -> TypeMapping {
    TypeMapping::new(
        id("type:component"),
        " Component ",
        InterpretationTargetKind::Cell,
        "component",
    )
    .expect("type mapping should be valid")
}

fn dependency_mapping(component_id: &Id) -> MorphismTypeMapping {
    MorphismTypeMapping::new(
        id("morphism-type:dependency"),
        "Dependency",
        "interpretation",
    )
    .expect("morphism mapping should be valid")
    .with_source_type_mapping(component_id.clone())
    .with_target_type_mapping(component_id.clone())
}

fn boundary_invariant(component_id: &Id) -> InvariantTemplate {
    InvariantTemplate::new(
        id("invariant:boundary"),
        "Boundary rule",
        "A component must respect declared ownership boundaries.",
    )
    .expect("invariant template should be valid")
    .with_type_mapping(component_id.clone())
    .with_parameter(
        TemplateParameter::required("owner")
            .expect("parameter should be valid")
            .with_description("Boundary owner label")
            .expect("description should be valid"),
    )
}

fn review_projection(component_id: &Id, invariant_id: &Id) -> ProjectionTemplate {
    ProjectionTemplate::new(
        id("projection:review-report"),
        "Review report",
        "architect",
        "review",
        "sections",
    )
    .expect("projection template should be valid")
    .with_source_type_mapping(component_id.clone())
    .with_invariant_template(invariant_id.clone())
}

fn notes_adapter(component_id: &Id, dependency_id: &Id) -> LiftAdapterDefinition {
    LiftAdapterDefinition::new(
        id("lift:architecture-notes"),
        "Architecture notes",
        "plain_text",
    )
    .expect("adapter should be valid")
    .with_supported_type_mapping(component_id.clone())
    .with_supported_morphism_type_mapping(dependency_id.clone())
}

fn registered_architecture_fixture() -> Fixture {
    let component_mapping = component_mapping();
    let dependency_mapping = dependency_mapping(&component_mapping.id);
    let invariant = boundary_invariant(&component_mapping.id);
    let projection = review_projection(&component_mapping.id, &invariant.id);
    let adapter = notes_adapter(&component_mapping.id, &dependency_mapping.id);
    let mut package = InterpretationPackage::new(
        id("package:architecture"),
        "Architecture interpretation",
        "0.1.0",
    )
    .expect("package should be valid")
    .with_metadata("product_family", "architecture")
    .expect("metadata should be valid");

    package
        .register_type_mapping(component_mapping.clone())
        .expect("type mapping registration should succeed");
    package
        .register_invariant_template(invariant.clone())
        .expect("invariant registration should succeed");
    package
        .register_morphism_type_mapping(dependency_mapping)
        .expect("morphism mapping registration should succeed");
    package
        .register_projection_template(projection.clone())
        .expect("projection registration should succeed");
    package
        .register_lift_adapter(adapter.clone())
        .expect("adapter registration should succeed");

    Fixture {
        package,
        component_mapping,
        invariant,
        projection,
        adapter,
    }
}

#[test]
fn package_registers_and_looks_up_interpretation_definitions() {
    let fixture = registered_architecture_fixture();

    assert_eq!(
        fixture.package.metadata_value("product_family"),
        Some("architecture")
    );
    assert_eq!(
        fixture
            .package
            .type_mappings_by_source_type("Component")
            .first()
            .map(|mapping| &mapping.id),
        Some(&fixture.component_mapping.id)
    );
    assert!(fixture
        .package
        .type_mappings_by_target_kind(&InterpretationTargetKind::Cell)
        .iter()
        .any(|mapping| mapping.id == fixture.component_mapping.id));
    assert_eq!(
        fixture
            .package
            .projection_template(&fixture.projection.id)
            .expect("projection should be registered")
            .invariant_template_ids,
        vec![fixture.invariant.id]
    );
    assert_eq!(
        fixture.package.lift_adapter(&fixture.adapter.id),
        Some(&fixture.adapter)
    );
}

#[test]
fn registration_rejects_duplicate_definition_ids() {
    let mut package = InterpretationPackage::new(id("package:test"), "Test", "0.1.0")
        .expect("package should be valid");
    let first = TypeMapping::new(
        id("definition:shared"),
        "Component",
        InterpretationTargetKind::Cell,
        "component",
    )
    .expect("mapping should be valid");
    let second = InvariantTemplate::new(
        first.id.clone(),
        "Boundary rule",
        "Components must respect boundaries.",
    )
    .expect("template should be valid");

    package
        .register_type_mapping(first)
        .expect("first registration should succeed");

    assert_eq!(
        package
            .register_invariant_template(second)
            .expect_err("duplicate ids should fail")
            .code(),
        "malformed_field"
    );
}

#[test]
fn registration_rejects_missing_references() {
    let mut package = InterpretationPackage::new(id("package:test"), "Test", "0.1.0")
        .expect("package should be valid");
    let projection = ProjectionTemplate::new(
        id("projection:missing"),
        "Missing invariant report",
        "architect",
        "review",
        "text",
    )
    .expect("projection should be valid")
    .with_invariant_template(id("invariant:missing"));

    assert_eq!(
        package
            .register_projection_template(projection)
            .expect_err("missing invariant reference should fail")
            .code(),
        "malformed_field"
    );
}

#[test]
fn constructors_normalize_required_text() {
    let mapping = TypeMapping::new(
        id("type:service"),
        " Service ",
        InterpretationTargetKind::Cell,
        " service ",
    )
    .expect("mapping should be valid")
    .with_metadata(" product_family ", " architecture ")
    .expect("metadata should be valid");

    assert_eq!(mapping.source_type, "Service");
    assert_eq!(mapping.target_type, "service");
    assert_eq!(
        mapping.metadata_value("product_family"),
        Some("architecture")
    );
    assert_eq!(
        TypeMapping::new(
            id("type:empty"),
            " ",
            InterpretationTargetKind::Cell,
            "component",
        )
        .expect_err("empty source type should fail")
        .code(),
        "malformed_field"
    );
}
