//! Reusable Architecture Product interpretation vocabulary.

use crate::{
    InterpretationPackage, InterpretationTargetKind, InvariantTemplate, LiftAdapterDefinition,
    MorphismTypeMapping, ProjectionTemplate, TemplateParameter, TypeMapping,
};
use higher_graphen_core::{CoreError, Id, Result};

const PACKAGE_ID: &str = "package:architecture-interpretation";
const PACKAGE_VERSION: &str = "0.1.0";

const TYPE_COMPONENT: &str = "type:architecture-component";
const TYPE_API: &str = "type:architecture-api";
const TYPE_DATABASE: &str = "type:architecture-database";
const TYPE_EVENT: &str = "type:architecture-event";
const TYPE_REQUIREMENT: &str = "type:architecture-requirement";
const TYPE_TEST: &str = "type:architecture-test";
const TYPE_OWNERSHIP: &str = "type:architecture-ownership";
const TYPE_ACCESS: &str = "type:architecture-access";

const MORPHISM_DEPENDENCY: &str = "morphism-type:architecture-dependency";
const MORPHISM_INTERFACE: &str = "morphism-type:architecture-interface";
const MORPHISM_REQUIREMENT_TO_DESIGN: &str = "morphism-type:architecture-requirement-to-design";
const MORPHISM_DESIGN_TO_TEST: &str = "morphism-type:architecture-design-to-test";
const MORPHISM_OWNERSHIP: &str = "morphism-type:architecture-ownership";
const MORPHISM_ACCESS: &str = "morphism-type:architecture-access";

const INVARIANT_NO_CROSS_CONTEXT_DB_ACCESS: &str =
    "invariant-template:architecture-no-cross-context-direct-database-access";
const INVARIANT_OWNERSHIP_RESOLVED: &str = "invariant-template:architecture-ownership-resolved";
const INVARIANT_REQUIREMENT_VERIFIED: &str =
    "invariant-template:architecture-requirement-must-be-verified";
const INVARIANT_PROJECTION_INFORMATION_LOSS: &str =
    "invariant-template:architecture-projection-declares-information-loss";

const PROJECTION_ARCHITECTURE_REVIEW: &str = "projection-template:architecture-review-report";
const PROJECTION_DESIGN_RISK_SUMMARY: &str = "projection-template:architecture-design-risk-summary";
const PROJECTION_DEVELOPER_ACTION_PLAN: &str =
    "projection-template:architecture-developer-action-plan";

const ADAPTER_ARCHITECTURE_INPUT_LIFT: &str = "lift-adapter:architecture-input-lift-v1";
const ARCHITECTURE_INPUT_KIND: &str = "highergraphen.architecture.input.v1";
const ARCHITECTURE_TYPED_GRAPH_OUTPUT_KIND: &str = "highergraphen.space.typed_graph.v1";
const ARCHITECTURE_INPUT_LIFT_REPORT_SCHEMA: &str =
    "highergraphen.architecture.input_lift.report.v1";

/// Builds the reusable Architecture Product interpretation package.
pub fn architecture_interpretation_package() -> Result<InterpretationPackage> {
    let mut package = InterpretationPackage::new(
        id(PACKAGE_ID)?,
        "Architecture interpretation package",
        PACKAGE_VERSION,
    )?
    .with_description(
        "Reusable Architecture Product vocabulary for components, interfaces, ownership, access, requirements, tests, and events.",
    )?
    .with_metadata("product_family", "architecture")?;

    for mapping in architecture_type_mappings()? {
        package.register_type_mapping(mapping)?;
    }
    for template in architecture_invariant_templates()? {
        package.register_invariant_template(template)?;
    }
    for mapping in architecture_morphism_type_mappings()? {
        package.register_morphism_type_mapping(mapping)?;
    }
    for template in architecture_projection_templates()? {
        package.register_projection_template(template)?;
    }
    for adapter in architecture_lift_adapters()? {
        package.register_lift_adapter(adapter)?;
    }

    Ok(package)
}

/// Resolves an Architecture Product component input type to a package type mapping.
pub fn architecture_component_type_mapping<'a>(
    package: &'a InterpretationPackage,
    component_type: &str,
) -> Result<&'a TypeMapping> {
    let mapping_id = component_type_mapping_id(component_type)?;
    package
        .type_mapping(&mapping_id)
        .ok_or_else(|| missing_package_definition("type_mapping_id", &mapping_id))
}

/// Resolves an Architecture Product relation input type to a package morphism mapping.
pub fn architecture_relation_morphism_type_mapping<'a>(
    package: &'a InterpretationPackage,
    relation_type: &str,
) -> Result<&'a MorphismTypeMapping> {
    let mapping_id = relation_morphism_type_mapping_id(relation_type)?;
    package
        .morphism_type_mapping(&mapping_id)
        .ok_or_else(|| missing_package_definition("morphism_type_mapping_id", &mapping_id))
}

/// Returns the Architecture Product bounded JSON input lift adapter definition.
pub fn architecture_input_lift_adapter(
    package: &InterpretationPackage,
) -> Result<&LiftAdapterDefinition> {
    lift_adapter(package, ADAPTER_ARCHITECTURE_INPUT_LIFT)
}

/// Resolves a component input type through the architecture input lift adapter.
pub fn architecture_input_lift_component_type_mapping<'a>(
    package: &'a InterpretationPackage,
    component_type: &str,
) -> Result<&'a TypeMapping> {
    let mapping = architecture_component_type_mapping(package, component_type)?;
    let adapter = architecture_input_lift_adapter(package)?;
    ensure_adapter_supports_type_mapping(adapter, &mapping.id)?;
    Ok(mapping)
}

/// Resolves a relation input type through the architecture input lift adapter.
pub fn architecture_input_lift_relation_morphism_type_mapping<'a>(
    package: &'a InterpretationPackage,
    relation_type: &str,
) -> Result<&'a MorphismTypeMapping> {
    let mapping = architecture_relation_morphism_type_mapping(package, relation_type)?;
    let adapter = architecture_input_lift_adapter(package)?;
    ensure_adapter_supports_morphism_type_mapping(adapter, &mapping.id)?;
    Ok(mapping)
}

/// Returns the boundary invariant template for direct cross-context database access.
pub fn architecture_boundary_invariant_template(
    package: &InterpretationPackage,
) -> Result<&InvariantTemplate> {
    invariant_template(package, INVARIANT_NO_CROSS_CONTEXT_DB_ACCESS)
}

/// Returns the ownership-resolution invariant template.
pub fn architecture_ownership_invariant_template(
    package: &InterpretationPackage,
) -> Result<&InvariantTemplate> {
    invariant_template(package, INVARIANT_OWNERSHIP_RESOLVED)
}

/// Returns the requirement verification invariant template.
pub fn architecture_requirement_verification_invariant_template(
    package: &InterpretationPackage,
) -> Result<&InvariantTemplate> {
    invariant_template(package, INVARIANT_REQUIREMENT_VERIFIED)
}

/// Returns the projection information-loss invariant template.
pub fn architecture_projection_information_loss_invariant_template(
    package: &InterpretationPackage,
) -> Result<&InvariantTemplate> {
    invariant_template(package, INVARIANT_PROJECTION_INFORMATION_LOSS)
}

/// Returns the architecture review report projection template.
pub fn architecture_review_projection_template(
    package: &InterpretationPackage,
) -> Result<&ProjectionTemplate> {
    projection_template(package, PROJECTION_ARCHITECTURE_REVIEW)
}

/// Returns the design risk summary projection template.
pub fn architecture_design_risk_summary_projection_template(
    package: &InterpretationPackage,
) -> Result<&ProjectionTemplate> {
    projection_template(package, PROJECTION_DESIGN_RISK_SUMMARY)
}

/// Returns the developer action plan projection template.
pub fn architecture_developer_action_plan_projection_template(
    package: &InterpretationPackage,
) -> Result<&ProjectionTemplate> {
    projection_template(package, PROJECTION_DEVELOPER_ACTION_PLAN)
}

fn architecture_type_mappings() -> Result<Vec<TypeMapping>> {
    Ok(vec![
        type_mapping(
            TYPE_COMPONENT,
            "Component",
            InterpretationTargetKind::Cell,
            "component",
            "Deployable or logical architecture element such as a service, module, or subsystem.",
            "component,service,module,subsystem",
        )?,
        type_mapping(
            TYPE_API,
            "API",
            InterpretationTargetKind::Cell,
            "api",
            "Interface surface exposed for synchronous or asynchronous interaction.",
            "api,endpoint,interface",
        )?,
        type_mapping(
            TYPE_DATABASE,
            "Database",
            InterpretationTargetKind::Cell,
            "database",
            "Persistent datastore, schema, table family, or external data source.",
            "database,db,datastore,table,schema",
        )?,
        type_mapping(
            TYPE_EVENT,
            "Event",
            InterpretationTargetKind::Cell,
            "event",
            "Published or consumed domain, integration, or infrastructure event.",
            "event,message,topic",
        )?,
        type_mapping(
            TYPE_REQUIREMENT,
            "Requirement",
            InterpretationTargetKind::Cell,
            "requirement",
            "Accepted requirement, constraint, or architectural decision requirement.",
            "requirement,constraint,adr_requirement",
        )?,
        type_mapping(
            TYPE_TEST,
            "Test",
            InterpretationTargetKind::Cell,
            "test",
            "Automated or accepted manual verification artifact.",
            "test,test_case,verification",
        )?,
        type_mapping(
            TYPE_OWNERSHIP,
            "Ownership",
            InterpretationTargetKind::Incidence,
            "ownership",
            "Relation declaring which component or context owns an architecture element.",
            "ownership,owns,owns_database,owns_api,owns_event",
        )?,
        type_mapping(
            TYPE_ACCESS,
            "Access",
            InterpretationTargetKind::Incidence,
            "access",
            "Relation declaring a direct read, write, call, publish, or consume path.",
            "access,reads_database,writes_database,calls_api,publishes_event,consumes_event",
        )?,
    ])
}

fn architecture_morphism_type_mappings() -> Result<Vec<MorphismTypeMapping>> {
    Ok(vec![
        MorphismTypeMapping::new(id(MORPHISM_DEPENDENCY)?, "Dependency", "dependency")?
            .with_source_type_mapping(id(TYPE_COMPONENT)?)
            .with_source_type_mapping(id(TYPE_API)?)
            .with_source_type_mapping(id(TYPE_EVENT)?)
            .with_target_type_mapping(id(TYPE_COMPONENT)?)
            .with_target_type_mapping(id(TYPE_API)?)
            .with_target_type_mapping(id(TYPE_EVENT)?)
            .with_description("Architecture dependency between components, APIs, or events.")?,
        MorphismTypeMapping::new(id(MORPHISM_INTERFACE)?, "Interface", "interface")?
            .with_source_type_mapping(id(TYPE_COMPONENT)?)
            .with_target_type_mapping(id(TYPE_API)?)
            .with_description("Contracted interaction surface between a component and an API.")?,
        MorphismTypeMapping::new(
            id(MORPHISM_REQUIREMENT_TO_DESIGN)?,
            "RequirementToDesign",
            "requirement_to_design",
        )?
        .with_source_type_mapping(id(TYPE_REQUIREMENT)?)
        .with_target_type_mapping(id(TYPE_COMPONENT)?)
        .with_target_type_mapping(id(TYPE_API)?)
        .with_preserved_invariant_template(id(INVARIANT_REQUIREMENT_VERIFIED)?)
        .with_description(
            "Trace from an accepted requirement to the design element that realizes it.",
        )?,
        MorphismTypeMapping::new(
            id(MORPHISM_DESIGN_TO_TEST)?,
            "DesignToTest",
            "design_to_test",
        )?
        .with_source_type_mapping(id(TYPE_COMPONENT)?)
        .with_source_type_mapping(id(TYPE_API)?)
        .with_target_type_mapping(id(TYPE_TEST)?)
        .with_preserved_invariant_template(id(INVARIANT_REQUIREMENT_VERIFIED)?)
        .with_description(
            "Trace from a design element to the test or verification that covers it.",
        )?,
        MorphismTypeMapping::new(id(MORPHISM_OWNERSHIP)?, "Ownership", "ownership")?
            .with_source_type_mapping(id(TYPE_COMPONENT)?)
            .with_target_type_mapping(id(TYPE_DATABASE)?)
            .with_target_type_mapping(id(TYPE_API)?)
            .with_target_type_mapping(id(TYPE_EVENT)?)
            .with_preserved_invariant_template(id(INVARIANT_OWNERSHIP_RESOLVED)?)
            .with_description(
                "Ownership relation for databases, APIs, events, and similar assets.",
            )?,
        MorphismTypeMapping::new(id(MORPHISM_ACCESS)?, "Access", "access")?
            .with_source_type_mapping(id(TYPE_COMPONENT)?)
            .with_target_type_mapping(id(TYPE_DATABASE)?)
            .with_target_type_mapping(id(TYPE_API)?)
            .with_target_type_mapping(id(TYPE_EVENT)?)
            .with_target_type_mapping(id(TYPE_COMPONENT)?)
            .with_preserved_invariant_template(id(INVARIANT_NO_CROSS_CONTEXT_DB_ACCESS)?)
            .with_description(
                "Runtime or design-time access relation between architecture elements.",
            )?,
    ])
}

fn architecture_invariant_templates() -> Result<Vec<InvariantTemplate>> {
    Ok(vec![
        no_cross_context_direct_database_access_invariant()?,
        ownership_resolved_invariant()?,
        requirement_must_be_verified_invariant()?,
        projection_declares_information_loss_invariant()?,
    ])
}

fn no_cross_context_direct_database_access_invariant() -> Result<InvariantTemplate> {
    InvariantTemplate::new(
        id(INVARIANT_NO_CROSS_CONTEXT_DB_ACCESS)?,
        "No cross-context direct database access",
        "A component must not directly access a database owned by another context.",
    )?
    .with_type_mapping(id(TYPE_COMPONENT)?)
    .with_type_mapping(id(TYPE_DATABASE)?)
    .with_type_mapping(id(TYPE_ACCESS)?)
    .with_type_mapping(id(TYPE_OWNERSHIP)?)
    .with_parameter(required_parameter(
        "source_context_id",
        "Context containing the accessing component.",
    )?)
    .with_parameter(required_parameter(
        "target_context_id",
        "Context owning the accessed database.",
    )?)
    .with_parameter(optional_parameter(
        "approved_exception_id",
        "Accepted exception or waiver that permits the direct access.",
    )?)
    .with_metadata("category", "boundary")
}

fn ownership_resolved_invariant() -> Result<InvariantTemplate> {
    InvariantTemplate::new(
        id(INVARIANT_OWNERSHIP_RESOLVED)?,
        "Ownership is resolved",
        "Every database, API, and event referenced by an architecture relation must have a declared owner.",
    )?
    .with_type_mapping(id(TYPE_COMPONENT)?)
    .with_type_mapping(id(TYPE_DATABASE)?)
    .with_type_mapping(id(TYPE_API)?)
    .with_type_mapping(id(TYPE_EVENT)?)
    .with_type_mapping(id(TYPE_OWNERSHIP)?)
    .with_parameter(required_parameter(
        "owned_element_id",
        "Architecture element whose owner must be known.",
    )?)
    .with_parameter(required_parameter(
        "owner_id",
        "Component or context responsible for the owned element.",
    )?)
    .with_metadata("category", "ownership")
}

fn requirement_must_be_verified_invariant() -> Result<InvariantTemplate> {
    InvariantTemplate::new(
        id(INVARIANT_REQUIREMENT_VERIFIED)?,
        "Requirement must be verified",
        "A requirement must map to at least one design element and at least one accepted test or verification method.",
    )?
    .with_type_mapping(id(TYPE_REQUIREMENT)?)
    .with_type_mapping(id(TYPE_COMPONENT)?)
    .with_type_mapping(id(TYPE_API)?)
    .with_type_mapping(id(TYPE_TEST)?)
    .with_parameter(required_parameter(
        "requirement_id",
        "Accepted requirement or constraint under review.",
    )?)
    .with_parameter(required_parameter(
        "design_element_id",
        "Component, API, event, or other design element that realizes the requirement.",
    )?)
    .with_parameter(required_parameter(
        "verification_id",
        "Test or accepted verification method covering the requirement.",
    )?)
    .with_metadata("category", "verification")
}

fn projection_declares_information_loss_invariant() -> Result<InvariantTemplate> {
    InvariantTemplate::new(
        id(INVARIANT_PROJECTION_INFORMATION_LOSS)?,
        "Projection declares information loss",
        "Every architecture projection must declare which source details were omitted or summarized.",
    )?
    .with_type_mapping(id(TYPE_COMPONENT)?)
    .with_type_mapping(id(TYPE_API)?)
    .with_type_mapping(id(TYPE_DATABASE)?)
    .with_type_mapping(id(TYPE_EVENT)?)
    .with_type_mapping(id(TYPE_REQUIREMENT)?)
    .with_type_mapping(id(TYPE_TEST)?)
    .with_parameter(required_parameter(
        "projection_id",
        "Projection whose information loss declaration is being checked.",
    )?)
    .with_parameter(required_parameter(
        "source_ids",
        "Source identifiers represented by the projection.",
    )?)
    .with_metadata("category", "projection_traceability")
}

fn architecture_projection_templates() -> Result<Vec<ProjectionTemplate>> {
    Ok(vec![
        architecture_review_report_projection()?,
        design_risk_summary_projection()?,
        developer_action_plan_projection()?,
    ])
}

fn architecture_review_report_projection() -> Result<ProjectionTemplate> {
    ProjectionTemplate::new(
        id(PROJECTION_ARCHITECTURE_REVIEW)?,
        "Architecture review report",
        "human",
        "architecture_review",
        "sections",
    )?
    .with_source_type_mapping(id(TYPE_COMPONENT)?)
    .with_source_type_mapping(id(TYPE_API)?)
    .with_source_type_mapping(id(TYPE_DATABASE)?)
    .with_source_type_mapping(id(TYPE_EVENT)?)
    .with_source_type_mapping(id(TYPE_REQUIREMENT)?)
    .with_source_type_mapping(id(TYPE_TEST)?)
    .with_invariant_template(id(INVARIANT_NO_CROSS_CONTEXT_DB_ACCESS)?)
    .with_invariant_template(id(INVARIANT_OWNERSHIP_RESOLVED)?)
    .with_invariant_template(id(INVARIANT_REQUIREMENT_VERIFIED)?)
    .with_invariant_template(id(INVARIANT_PROJECTION_INFORMATION_LOSS)?)
    .with_parameter(optional_parameter(
        "focus_context_id",
        "Optional bounded context to emphasize in the review.",
    )?)
    .with_metadata(
        "section_order",
        "summary,invariants,obstructions,completion_candidates,recommended_actions",
    )
}

fn design_risk_summary_projection() -> Result<ProjectionTemplate> {
    ProjectionTemplate::new(
        id(PROJECTION_DESIGN_RISK_SUMMARY)?,
        "Design risk summary",
        "human",
        "design_risk_summary",
        "risk_table",
    )?
    .with_source_type_mapping(id(TYPE_COMPONENT)?)
    .with_source_type_mapping(id(TYPE_API)?)
    .with_source_type_mapping(id(TYPE_DATABASE)?)
    .with_source_type_mapping(id(TYPE_EVENT)?)
    .with_invariant_template(id(INVARIANT_NO_CROSS_CONTEXT_DB_ACCESS)?)
    .with_invariant_template(id(INVARIANT_OWNERSHIP_RESOLVED)?)
    .with_invariant_template(id(INVARIANT_PROJECTION_INFORMATION_LOSS)?)
    .with_parameter(optional_parameter(
        "minimum_severity",
        "Minimum risk severity to include.",
    )?)
    .with_metadata("section_order", "risk,impact,evidence,missing_structure")
}

fn developer_action_plan_projection() -> Result<ProjectionTemplate> {
    ProjectionTemplate::new(
        id(PROJECTION_DEVELOPER_ACTION_PLAN)?,
        "Developer action plan",
        "human",
        "developer_action_plan",
        "checklist",
    )?
    .with_source_type_mapping(id(TYPE_COMPONENT)?)
    .with_source_type_mapping(id(TYPE_API)?)
    .with_source_type_mapping(id(TYPE_DATABASE)?)
    .with_source_type_mapping(id(TYPE_TEST)?)
    .with_invariant_template(id(INVARIANT_REQUIREMENT_VERIFIED)?)
    .with_invariant_template(id(INVARIANT_PROJECTION_INFORMATION_LOSS)?)
    .with_parameter(optional_parameter(
        "assignee",
        "Developer, team, or owning context for the action list.",
    )?)
    .with_metadata("section_order", "action,reason,source_ids,verification")
}

fn architecture_lift_adapters() -> Result<Vec<LiftAdapterDefinition>> {
    Ok(vec![LiftAdapterDefinition::new(
        id(ADAPTER_ARCHITECTURE_INPUT_LIFT)?,
        "Architecture input lift v1",
        ARCHITECTURE_INPUT_KIND,
    )?
    .with_output_kind(ARCHITECTURE_TYPED_GRAPH_OUTPUT_KIND)?
    .with_supported_type_mapping(id(TYPE_COMPONENT)?)
    .with_supported_type_mapping(id(TYPE_API)?)
    .with_supported_type_mapping(id(TYPE_DATABASE)?)
    .with_supported_type_mapping(id(TYPE_EVENT)?)
    .with_supported_type_mapping(id(TYPE_REQUIREMENT)?)
    .with_supported_type_mapping(id(TYPE_TEST)?)
    .with_supported_type_mapping(id(TYPE_OWNERSHIP)?)
    .with_supported_type_mapping(id(TYPE_ACCESS)?)
    .with_supported_morphism_type_mapping(id(MORPHISM_DEPENDENCY)?)
    .with_supported_morphism_type_mapping(id(MORPHISM_INTERFACE)?)
    .with_supported_morphism_type_mapping(id(MORPHISM_REQUIREMENT_TO_DESIGN)?)
    .with_supported_morphism_type_mapping(id(MORPHISM_DESIGN_TO_TEST)?)
    .with_supported_morphism_type_mapping(id(MORPHISM_OWNERSHIP)?)
    .with_supported_morphism_type_mapping(id(MORPHISM_ACCESS)?)
    .with_metadata(
        "report_schema",
        ARCHITECTURE_INPUT_LIFT_REPORT_SCHEMA,
    )?])
}

fn type_mapping(
    id_value: &str,
    source_type: &str,
    target_kind: InterpretationTargetKind,
    target_type: &str,
    description: &str,
    input_aliases: &str,
) -> Result<TypeMapping> {
    TypeMapping::new(id(id_value)?, source_type, target_kind, target_type)?
        .with_description(description)?
        .with_metadata("input_aliases", input_aliases)
}

fn required_parameter(name: &str, description: &str) -> Result<TemplateParameter> {
    TemplateParameter::required(name)?.with_description(description)
}

fn optional_parameter(name: &str, description: &str) -> Result<TemplateParameter> {
    TemplateParameter::optional(name)?.with_description(description)
}

fn invariant_template<'a>(
    package: &'a InterpretationPackage,
    template_id: &str,
) -> Result<&'a InvariantTemplate> {
    let template_id = id(template_id)?;
    package
        .invariant_template(&template_id)
        .ok_or_else(|| missing_package_definition("invariant_template_id", &template_id))
}

fn projection_template<'a>(
    package: &'a InterpretationPackage,
    template_id: &str,
) -> Result<&'a ProjectionTemplate> {
    let template_id = id(template_id)?;
    package
        .projection_template(&template_id)
        .ok_or_else(|| missing_package_definition("projection_template_id", &template_id))
}

fn lift_adapter<'a>(
    package: &'a InterpretationPackage,
    adapter_id: &str,
) -> Result<&'a LiftAdapterDefinition> {
    let adapter_id = id(adapter_id)?;
    package
        .lift_adapter(&adapter_id)
        .ok_or_else(|| missing_package_definition("lift_adapter_id", &adapter_id))
}

fn ensure_adapter_supports_type_mapping(
    adapter: &LiftAdapterDefinition,
    mapping_id: &Id,
) -> Result<()> {
    if adapter.supported_type_mapping_ids.contains(mapping_id) {
        Ok(())
    } else {
        Err(unsupported_adapter_mapping(
            "type_mapping_id",
            adapter,
            mapping_id,
        ))
    }
}

fn ensure_adapter_supports_morphism_type_mapping(
    adapter: &LiftAdapterDefinition,
    mapping_id: &Id,
) -> Result<()> {
    if adapter
        .supported_morphism_type_mapping_ids
        .contains(mapping_id)
    {
        Ok(())
    } else {
        Err(unsupported_adapter_mapping(
            "morphism_type_mapping_id",
            adapter,
            mapping_id,
        ))
    }
}

fn component_type_mapping_id(component_type: &str) -> Result<Id> {
    let normalized = normalized_token(component_type);
    let id_value = match normalized.as_str() {
        "component" | "service" | "module" | "subsystem" => TYPE_COMPONENT,
        "api" | "endpoint" | "interface" => TYPE_API,
        "database" | "db" | "datastore" | "table" | "schema" => TYPE_DATABASE,
        "event" | "message" | "topic" => TYPE_EVENT,
        "requirement" | "constraint" | "adr_requirement" => TYPE_REQUIREMENT,
        "test" | "test_case" | "verification" => TYPE_TEST,
        _ => {
            return Err(unsupported_architecture_type(
                "component_type",
                component_type,
            ));
        }
    };
    id(id_value)
}

fn relation_morphism_type_mapping_id(relation_type: &str) -> Result<Id> {
    let normalized = normalized_token(relation_type);
    let id_value = match normalized.as_str() {
        "depends_on" | "depends" | "publishes_event" | "consumes_event" => MORPHISM_DEPENDENCY,
        "interface" | "calls_api" | "implements_api" | "exposes_api" => MORPHISM_INTERFACE,
        "realizes_requirement" | "requirement_to_design" => MORPHISM_REQUIREMENT_TO_DESIGN,
        "covered_by_test" | "design_to_test" | "verified_by_test" => MORPHISM_DESIGN_TO_TEST,
        "ownership" | "owns" | "owns_database" | "owns_api" | "owns_event" => MORPHISM_OWNERSHIP,
        "access" | "reads_database" | "writes_database" | "accesses_database" => MORPHISM_ACCESS,
        _ => {
            return Err(unsupported_architecture_type(
                "relation_type",
                relation_type,
            ));
        }
    };
    id(id_value)
}

fn normalized_token(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace([' ', '-'], "_")
}

fn id(value: &str) -> Result<Id> {
    Id::new(value)
}

fn missing_package_definition(field: &str, id: &Id) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: format!("architecture package does not contain referenced identifier {id}"),
    }
}

fn unsupported_architecture_type(field: &str, value: &str) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: format!("unsupported architecture {field} {value:?}"),
    }
}

fn unsupported_adapter_mapping(
    field: &str,
    adapter: &LiftAdapterDefinition,
    mapping_id: &Id,
) -> CoreError {
    CoreError::MalformedField {
        field: field.to_owned(),
        reason: format!(
            "adapter {} does not support mapping {mapping_id}",
            adapter.id
        ),
    }
}
