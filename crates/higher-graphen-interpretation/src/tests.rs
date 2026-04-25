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

#[test]
fn architecture_package_defines_product_vocabulary_mappings() {
    let package = architecture::architecture_interpretation_package()
        .expect("architecture package should build");

    assert_eq!(
        package.metadata_value("product_family"),
        Some("architecture")
    );
    assert_eq!(package.type_mappings.len(), 8);
    assert_eq!(package.morphism_type_mappings.len(), 6);
    assert_eq!(package.invariant_templates.len(), 4);
    assert_eq!(package.projection_templates.len(), 3);
    assert_eq!(package.lift_adapters.len(), 1);

    let service = architecture::architecture_component_type_mapping(&package, " service ")
        .expect("service should resolve as an architecture component");
    assert_eq!(service.source_type, "Component");
    assert_eq!(service.target_kind, InterpretationTargetKind::Cell);
    assert_eq!(service.target_type, "component");

    let database = architecture::architecture_component_type_mapping(&package, "database")
        .expect("database should resolve");
    assert_eq!(database.source_type, "Database");
    assert_eq!(database.target_type, "database");

    let access =
        architecture::architecture_relation_morphism_type_mapping(&package, "reads_database")
            .expect("direct database read should resolve as access");
    assert_eq!(access.source_type, "Access");
    assert_eq!(access.morphism_type, "access");

    let ownership =
        architecture::architecture_relation_morphism_type_mapping(&package, "owns_database")
            .expect("ownership relation should resolve");
    assert_eq!(ownership.source_type, "Ownership");
    assert_eq!(ownership.morphism_type, "ownership");
}

#[test]
fn architecture_package_defines_input_lift_adapter() {
    let package = architecture::architecture_interpretation_package()
        .expect("architecture package should build");

    let adapter = architecture::architecture_input_lift_adapter(&package)
        .expect("architecture input lift adapter should be registered");
    assert_eq!(adapter.name, "Architecture input lift v1");
    assert_eq!(adapter.input_kind, "highergraphen.architecture.input.v1");
    assert_eq!(
        adapter.output_kind.as_deref(),
        Some("highergraphen.space.typed_graph.v1")
    );
    assert_eq!(
        adapter.metadata_value("report_schema"),
        Some("highergraphen.architecture.input_lift.report.v1")
    );
    assert_eq!(adapter.supported_type_mapping_ids.len(), 8);
    assert_eq!(adapter.supported_morphism_type_mapping_ids.len(), 6);

    let service = architecture::architecture_input_lift_component_type_mapping(&package, "service")
        .expect("service should resolve through lift adapter");
    assert_eq!(service.target_type, "component");
    let access = architecture::architecture_input_lift_relation_morphism_type_mapping(
        &package,
        "reads_database",
    )
    .expect("direct access should resolve through lift adapter");
    assert_eq!(access.morphism_type, "access");
}

#[test]
fn architecture_vocabulary_resolves_api_events_requirements_and_tests() {
    let package = architecture::architecture_interpretation_package()
        .expect("architecture package should build");

    let cases = [
        ("api", "API", "api"),
        ("topic", "Event", "event"),
        ("constraint", "Requirement", "requirement"),
        ("test_case", "Test", "test"),
    ];

    for (input_type, expected_source, expected_target) in cases {
        let mapping = architecture::architecture_component_type_mapping(&package, input_type)
            .expect("component type should resolve");
        assert_eq!(mapping.source_type, expected_source);
        assert_eq!(mapping.target_type, expected_target);
    }

    let interface =
        architecture::architecture_relation_morphism_type_mapping(&package, "calls_api")
            .expect("API calls should resolve as interface morphisms");
    assert_eq!(interface.morphism_type, "interface");
    let requirement_trace =
        architecture::architecture_relation_morphism_type_mapping(&package, "realizes_requirement")
            .expect("requirement trace should resolve");
    assert_eq!(requirement_trace.morphism_type, "requirement_to_design");
}

#[test]
fn architecture_vocabulary_rejects_unknown_input_types() {
    let package = architecture::architecture_interpretation_package()
        .expect("architecture package should build");

    assert_eq!(
        architecture::architecture_component_type_mapping(&package, "queue")
            .expect_err("unsupported component type should fail")
            .code(),
        "malformed_field"
    );
    assert_eq!(
        architecture::architecture_relation_morphism_type_mapping(&package, "mirrors")
            .expect_err("unsupported relation type should fail")
            .code(),
        "malformed_field"
    );
}

#[test]
fn architecture_package_defines_invariant_templates() {
    let package = architecture::architecture_interpretation_package()
        .expect("architecture package should build");

    let boundary = architecture::architecture_boundary_invariant_template(&package)
        .expect("boundary invariant should be registered");
    assert_eq!(boundary.name, "No cross-context direct database access");
    assert_eq!(boundary.metadata_value("category"), Some("boundary"));
    assert!(boundary.parameter("source_context_id").is_some());
    assert!(boundary.parameter("target_context_id").is_some());

    let ownership = architecture::architecture_ownership_invariant_template(&package)
        .expect("ownership invariant should be registered");
    assert_eq!(ownership.name, "Ownership is resolved");
    assert_eq!(ownership.metadata_value("category"), Some("ownership"));

    let requirement =
        architecture::architecture_requirement_verification_invariant_template(&package)
            .expect("requirement invariant should be registered");
    assert_eq!(requirement.name, "Requirement must be verified");
    assert!(requirement.parameter("verification_id").is_some());

    let projection =
        architecture::architecture_projection_information_loss_invariant_template(&package)
            .expect("projection loss invariant should be registered");
    assert_eq!(projection.name, "Projection declares information loss");
    assert!(projection.parameter("source_ids").is_some());

    let access =
        architecture::architecture_relation_morphism_type_mapping(&package, "reads_database")
            .expect("access mapping should resolve");
    assert_eq!(
        access.preserved_invariant_template_ids,
        vec![boundary.id.clone()]
    );
}

#[test]
fn architecture_package_defines_projection_templates() {
    let package = architecture::architecture_interpretation_package()
        .expect("architecture package should build");

    let review = architecture::architecture_review_projection_template(&package)
        .expect("review projection should be registered");
    assert_eq!(review.name, "Architecture review report");
    assert_eq!(review.audience, "human");
    assert_eq!(review.purpose, "architecture_review");
    assert_eq!(review.output_shape, "sections");
    assert_eq!(
        review.metadata_value("section_order"),
        Some("summary,invariants,obstructions,completion_candidates,recommended_actions")
    );
    assert_eq!(review.invariant_template_ids.len(), 4);

    let risk = architecture::architecture_design_risk_summary_projection_template(&package)
        .expect("risk projection should be registered");
    assert_eq!(risk.name, "Design risk summary");
    assert_eq!(risk.purpose, "design_risk_summary");

    let action = architecture::architecture_developer_action_plan_projection_template(&package)
        .expect("action plan projection should be registered");
    assert_eq!(action.name, "Developer action plan");
    assert_eq!(action.output_shape, "checklist");
}
