use super::{
    insert_metadata, malformed_field, metadata_value, required_text, InterpretationTargetKind,
    InvariantTemplate, LiftAdapterDefinition, Metadata, MorphismTypeMapping, ProjectionTemplate,
    TemplateParameter, TypeMapping,
};
use higher_graphen_core::{CoreError, Id, Provenance, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

/// Reusable package of interpretation definitions.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InterpretationPackage {
    /// Package identifier.
    id: Id,
    /// Human-readable package name.
    name: String,
    /// Package version label.
    version: String,
    /// Optional human-readable package description.
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Domain type mappings in this package.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    type_mappings: Vec<TypeMapping>,
    /// Domain morphism type mappings in this package.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    morphism_type_mappings: Vec<MorphismTypeMapping>,
    /// Invariant templates in this package.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    invariant_templates: Vec<InvariantTemplate>,
    /// Projection templates in this package.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    projection_templates: Vec<ProjectionTemplate>,
    /// Lift adapters in this package.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    lift_adapters: Vec<LiftAdapterDefinition>,
    /// Generic string metadata for product or tool hints.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    metadata: Metadata,
    /// Optional source and review metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    provenance: Option<Provenance>,
}

impl InterpretationPackage {
    /// Creates an empty interpretation package.
    pub fn new(id: Id, name: impl Into<String>, version: impl Into<String>) -> Result<Self> {
        Ok(Self {
            id,
            name: required_text("name", name)?,
            version: required_text("version", version)?,
            description: None,
            type_mappings: Vec::new(),
            morphism_type_mappings: Vec::new(),
            invariant_templates: Vec::new(),
            projection_templates: Vec::new(),
            lift_adapters: Vec::new(),
            metadata: Metadata::new(),
            provenance: None,
        })
    }

    /// Returns this package with a non-empty description.
    pub fn with_description(mut self, description: impl Into<String>) -> Result<Self> {
        self.description = Some(required_text("description", description)?);
        Ok(self)
    }

    /// Returns this package with one generic metadata entry.
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self> {
        insert_metadata(&mut self.metadata, key, value)?;
        Ok(self)
    }

    /// Returns this package with source and review metadata.
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Registers a type mapping after checking definition ID uniqueness.
    pub fn register_type_mapping(&mut self, mapping: TypeMapping) -> Result<()> {
        let mapping = normalize_type_mapping(mapping)?;
        validate_type_mapping(&mapping)?;
        self.ensure_definition_absent(&mapping.id)?;
        self.type_mappings.push(mapping);
        Ok(())
    }

    /// Registers a morphism type mapping after checking references.
    pub fn register_morphism_type_mapping(&mut self, mapping: MorphismTypeMapping) -> Result<()> {
        let mapping = normalize_morphism_type_mapping(mapping)?;
        validate_morphism_type_mapping(&mapping)?;
        self.ensure_definition_absent(&mapping.id)?;
        self.ensure_type_mapping_ids_exist(&mapping.source_type_mapping_ids)?;
        self.ensure_type_mapping_ids_exist(&mapping.target_type_mapping_ids)?;
        self.ensure_invariant_template_ids_exist(&mapping.preserved_invariant_template_ids)?;
        self.morphism_type_mappings.push(mapping);
        Ok(())
    }

    /// Registers an invariant template after checking referenced type mappings.
    pub fn register_invariant_template(&mut self, template: InvariantTemplate) -> Result<()> {
        let template = normalize_invariant_template(template)?;
        validate_invariant_template(&template)?;
        self.ensure_definition_absent(&template.id)?;
        self.ensure_type_mapping_ids_exist(&template.applies_to_type_mapping_ids)?;
        self.invariant_templates.push(template);
        Ok(())
    }

    /// Registers a projection template after checking referenced mappings and templates.
    pub fn register_projection_template(&mut self, template: ProjectionTemplate) -> Result<()> {
        let template = normalize_projection_template(template)?;
        validate_projection_template(&template)?;
        self.ensure_definition_absent(&template.id)?;
        self.ensure_type_mapping_ids_exist(&template.source_type_mapping_ids)?;
        self.ensure_invariant_template_ids_exist(&template.invariant_template_ids)?;
        self.projection_templates.push(template);
        Ok(())
    }

    /// Registers a lift adapter after checking referenced mappings.
    pub fn register_lift_adapter(&mut self, adapter: LiftAdapterDefinition) -> Result<()> {
        let adapter = normalize_lift_adapter(adapter)?;
        validate_lift_adapter(&adapter)?;
        self.ensure_definition_absent(&adapter.id)?;
        self.ensure_type_mapping_ids_exist(&adapter.supported_type_mapping_ids)?;
        self.ensure_morphism_type_mapping_ids_exist(&adapter.supported_morphism_type_mapping_ids)?;
        self.lift_adapters.push(adapter);
        Ok(())
    }

    /// Returns a type mapping by identifier.
    #[must_use]
    pub fn type_mapping(&self, id: &Id) -> Option<&TypeMapping> {
        self.type_mappings.iter().find(|mapping| mapping.id == *id)
    }

    /// Returns type mappings with a matching domain source type.
    #[must_use]
    pub fn type_mappings_by_source_type(&self, source_type: &str) -> Vec<&TypeMapping> {
        let normalized = source_type.trim();
        self.type_mappings
            .iter()
            .filter(|mapping| mapping.source_type == normalized)
            .collect()
    }

    /// Returns type mappings with a matching target category.
    #[must_use]
    pub fn type_mappings_by_target_kind(
        &self,
        target_kind: &InterpretationTargetKind,
    ) -> Vec<&TypeMapping> {
        self.type_mappings
            .iter()
            .filter(|mapping| &mapping.target_kind == target_kind)
            .collect()
    }

    /// Returns a morphism type mapping by identifier.
    #[must_use]
    pub fn morphism_type_mapping(&self, id: &Id) -> Option<&MorphismTypeMapping> {
        self.morphism_type_mappings
            .iter()
            .find(|mapping| mapping.id == *id)
    }

    /// Returns morphism type mappings with a matching domain source type.
    #[must_use]
    pub fn morphism_type_mappings_by_source_type(
        &self,
        source_type: &str,
    ) -> Vec<&MorphismTypeMapping> {
        let normalized = source_type.trim();
        self.morphism_type_mappings
            .iter()
            .filter(|mapping| mapping.source_type == normalized)
            .collect()
    }

    /// Returns an invariant template by identifier.
    #[must_use]
    pub fn invariant_template(&self, id: &Id) -> Option<&InvariantTemplate> {
        self.invariant_templates
            .iter()
            .find(|template| template.id == *id)
    }

    /// Returns a projection template by identifier.
    #[must_use]
    pub fn projection_template(&self, id: &Id) -> Option<&ProjectionTemplate> {
        self.projection_templates
            .iter()
            .find(|template| template.id == *id)
    }

    /// Returns a lift adapter by identifier.
    #[must_use]
    pub fn lift_adapter(&self, id: &Id) -> Option<&LiftAdapterDefinition> {
        self.lift_adapters.iter().find(|adapter| adapter.id == *id)
    }

    /// Returns a package metadata value by key after trimming the lookup key.
    #[must_use]
    pub fn metadata_value(&self, key: &str) -> Option<&str> {
        metadata_value(&self.metadata, key)
    }

    /// Returns the package identifier.
    #[must_use]
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Returns the package name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the package version label.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the optional package description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the registered domain type mappings.
    #[must_use]
    pub fn type_mappings(&self) -> &[TypeMapping] {
        &self.type_mappings
    }

    /// Returns the registered morphism type mappings.
    #[must_use]
    pub fn morphism_type_mappings(&self) -> &[MorphismTypeMapping] {
        &self.morphism_type_mappings
    }

    /// Returns the registered invariant templates.
    #[must_use]
    pub fn invariant_templates(&self) -> &[InvariantTemplate] {
        &self.invariant_templates
    }

    /// Returns the registered projection templates.
    #[must_use]
    pub fn projection_templates(&self) -> &[ProjectionTemplate] {
        &self.projection_templates
    }

    /// Returns the registered lift adapters.
    #[must_use]
    pub fn lift_adapters(&self) -> &[LiftAdapterDefinition] {
        &self.lift_adapters
    }

    /// Returns package metadata.
    #[must_use]
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Returns optional source and review metadata.
    #[must_use]
    pub fn provenance(&self) -> Option<&Provenance> {
        self.provenance.as_ref()
    }

    fn ensure_definition_absent(&self, id: &Id) -> Result<()> {
        if self.contains_definition_id(id) {
            Err(malformed_field(
                "definition_id",
                format!("identifier {id} is already registered in this package"),
            ))
        } else {
            Ok(())
        }
    }

    fn contains_definition_id(&self, id: &Id) -> bool {
        self.type_mapping(id).is_some()
            || self.morphism_type_mapping(id).is_some()
            || self.invariant_template(id).is_some()
            || self.projection_template(id).is_some()
            || self.lift_adapter(id).is_some()
    }

    fn ensure_type_mapping_ids_exist(&self, ids: &[Id]) -> Result<()> {
        for id in ids {
            if self.type_mapping(id).is_none() {
                return Err(missing_reference("type_mapping_id", id));
            }
        }
        Ok(())
    }

    fn ensure_morphism_type_mapping_ids_exist(&self, ids: &[Id]) -> Result<()> {
        for id in ids {
            if self.morphism_type_mapping(id).is_none() {
                return Err(missing_reference("morphism_type_mapping_id", id));
            }
        }
        Ok(())
    }

    fn ensure_invariant_template_ids_exist(&self, ids: &[Id]) -> Result<()> {
        for id in ids {
            if self.invariant_template(id).is_none() {
                return Err(missing_reference("invariant_template_id", id));
            }
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for InterpretationPackage {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            id: Id,
            name: String,
            version: String,
            description: Option<String>,
            #[serde(default)]
            type_mappings: Vec<TypeMapping>,
            #[serde(default)]
            morphism_type_mappings: Vec<MorphismTypeMapping>,
            #[serde(default)]
            invariant_templates: Vec<InvariantTemplate>,
            #[serde(default)]
            projection_templates: Vec<ProjectionTemplate>,
            #[serde(default)]
            lift_adapters: Vec<LiftAdapterDefinition>,
            #[serde(default)]
            metadata: Metadata,
            provenance: Option<Provenance>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let mut package =
            Self::new(wire.id, wire.name, wire.version).map_err(serde::de::Error::custom)?;

        if let Some(description) = wire.description {
            package = package
                .with_description(description)
                .map_err(serde::de::Error::custom)?;
        }
        for (key, value) in wire.metadata {
            package = package
                .with_metadata(key, value)
                .map_err(serde::de::Error::custom)?;
        }
        package.provenance = wire.provenance;

        for mapping in wire.type_mappings {
            package
                .register_type_mapping(mapping)
                .map_err(serde::de::Error::custom)?;
        }
        for template in wire.invariant_templates {
            package
                .register_invariant_template(template)
                .map_err(serde::de::Error::custom)?;
        }
        for mapping in wire.morphism_type_mappings {
            package
                .register_morphism_type_mapping(mapping)
                .map_err(serde::de::Error::custom)?;
        }
        for template in wire.projection_templates {
            package
                .register_projection_template(template)
                .map_err(serde::de::Error::custom)?;
        }
        for adapter in wire.lift_adapters {
            package
                .register_lift_adapter(adapter)
                .map_err(serde::de::Error::custom)?;
        }

        Ok(package)
    }
}

fn normalize_metadata(metadata: Metadata) -> Result<Metadata> {
    let mut normalized = Metadata::new();
    for (key, value) in metadata {
        insert_metadata(&mut normalized, key, value)?;
    }
    Ok(normalized)
}

fn normalize_target_kind(
    target_kind: InterpretationTargetKind,
) -> Result<InterpretationTargetKind> {
    match target_kind {
        InterpretationTargetKind::Custom(extension) => InterpretationTargetKind::custom(extension),
        InterpretationTargetKind::Cell
        | InterpretationTargetKind::Incidence
        | InterpretationTargetKind::Complex
        | InterpretationTargetKind::Morphism
        | InterpretationTargetKind::Invariant
        | InterpretationTargetKind::Constraint
        | InterpretationTargetKind::Obstruction
        | InterpretationTargetKind::CompletionCandidate
        | InterpretationTargetKind::Projection => Ok(target_kind),
    }
}

fn normalize_template_parameter(parameter: TemplateParameter) -> Result<TemplateParameter> {
    let mut normalized = if parameter.required {
        TemplateParameter::required(parameter.name)?
    } else {
        TemplateParameter::optional(parameter.name)?
    };

    if let Some(description) = parameter.description {
        normalized = normalized.with_description(description)?;
    }
    if let Some(default_value) = parameter.default_value {
        normalized = normalized.with_default_value(default_value)?;
    }

    Ok(normalized)
}

fn normalize_type_mapping(mapping: TypeMapping) -> Result<TypeMapping> {
    let source_type = required_text("source_type", mapping.source_type)?;
    let target_type = required_text("target_type", mapping.target_type)?;
    let target_kind = normalize_target_kind(mapping.target_kind)?;
    let mut normalized = TypeMapping::new(mapping.id, source_type, target_kind, target_type)?;

    if let Some(description) = mapping.description {
        normalized = normalized.with_description(description)?;
    }
    normalized.metadata = normalize_metadata(mapping.metadata)?;
    normalized.provenance = mapping.provenance;

    Ok(normalized)
}

fn normalize_morphism_type_mapping(mapping: MorphismTypeMapping) -> Result<MorphismTypeMapping> {
    let mut normalized =
        MorphismTypeMapping::new(mapping.id, mapping.source_type, mapping.morphism_type)?;

    for id in mapping.source_type_mapping_ids {
        normalized = normalized.with_source_type_mapping(id);
    }
    for id in mapping.target_type_mapping_ids {
        normalized = normalized.with_target_type_mapping(id);
    }
    for id in mapping.preserved_invariant_template_ids {
        normalized = normalized.with_preserved_invariant_template(id);
    }
    if let Some(description) = mapping.description {
        normalized = normalized.with_description(description)?;
    }
    normalized.metadata = normalize_metadata(mapping.metadata)?;
    normalized.provenance = mapping.provenance;

    Ok(normalized)
}

fn normalize_invariant_template(template: InvariantTemplate) -> Result<InvariantTemplate> {
    let mut normalized = InvariantTemplate::new(template.id, template.name, template.statement)?;

    for parameter in template.parameters {
        normalized = normalized.with_parameter(normalize_template_parameter(parameter)?);
    }
    for id in template.applies_to_type_mapping_ids {
        normalized = normalized.with_type_mapping(id);
    }
    normalized.metadata = normalize_metadata(template.metadata)?;
    normalized.provenance = template.provenance;

    Ok(normalized)
}

fn normalize_projection_template(template: ProjectionTemplate) -> Result<ProjectionTemplate> {
    let mut normalized = ProjectionTemplate::new(
        template.id,
        template.name,
        template.audience,
        template.purpose,
        template.output_shape,
    )?;

    for parameter in template.parameters {
        normalized = normalized.with_parameter(normalize_template_parameter(parameter)?);
    }
    for id in template.source_type_mapping_ids {
        normalized = normalized.with_source_type_mapping(id);
    }
    for id in template.invariant_template_ids {
        normalized = normalized.with_invariant_template(id);
    }
    normalized.metadata = normalize_metadata(template.metadata)?;
    normalized.provenance = template.provenance;

    Ok(normalized)
}

fn normalize_lift_adapter(adapter: LiftAdapterDefinition) -> Result<LiftAdapterDefinition> {
    let mut normalized = LiftAdapterDefinition::new(adapter.id, adapter.name, adapter.input_kind)?;

    if let Some(output_kind) = adapter.output_kind {
        normalized = normalized.with_output_kind(output_kind)?;
    }
    for id in adapter.supported_type_mapping_ids {
        normalized = normalized.with_supported_type_mapping(id);
    }
    for id in adapter.supported_morphism_type_mapping_ids {
        normalized = normalized.with_supported_morphism_type_mapping(id);
    }
    normalized.metadata = normalize_metadata(adapter.metadata)?;
    normalized.provenance = adapter.provenance;

    Ok(normalized)
}

fn validate_type_mapping(mapping: &TypeMapping) -> Result<()> {
    required_text("source_type", mapping.source_type.as_str())?;
    required_text("target_type", mapping.target_type.as_str())?;
    validate_target_kind(&mapping.target_kind)
}

fn validate_target_kind(target_kind: &InterpretationTargetKind) -> Result<()> {
    match target_kind {
        InterpretationTargetKind::Custom(extension) => {
            required_text("target_kind", extension.as_str())?;
        }
        InterpretationTargetKind::Cell
        | InterpretationTargetKind::Incidence
        | InterpretationTargetKind::Complex
        | InterpretationTargetKind::Morphism
        | InterpretationTargetKind::Invariant
        | InterpretationTargetKind::Constraint
        | InterpretationTargetKind::Obstruction
        | InterpretationTargetKind::CompletionCandidate
        | InterpretationTargetKind::Projection => {}
    }
    Ok(())
}

fn validate_morphism_type_mapping(mapping: &MorphismTypeMapping) -> Result<()> {
    required_text("source_type", mapping.source_type.as_str())?;
    required_text("morphism_type", mapping.morphism_type.as_str())?;
    Ok(())
}

fn validate_invariant_template(template: &InvariantTemplate) -> Result<()> {
    required_text("name", template.name.as_str())?;
    required_text("statement", template.statement.as_str())?;
    validate_template_parameters(&template.parameters)
}

fn validate_projection_template(template: &ProjectionTemplate) -> Result<()> {
    required_text("name", template.name.as_str())?;
    required_text("audience", template.audience.as_str())?;
    required_text("purpose", template.purpose.as_str())?;
    required_text("output_shape", template.output_shape.as_str())?;
    validate_template_parameters(&template.parameters)
}

fn validate_lift_adapter(adapter: &LiftAdapterDefinition) -> Result<()> {
    required_text("name", adapter.name.as_str())?;
    required_text("input_kind", adapter.input_kind.as_str())?;
    if let Some(output_kind) = &adapter.output_kind {
        required_text("output_kind", output_kind.as_str())?;
    }
    Ok(())
}

fn validate_template_parameters(parameters: &[TemplateParameter]) -> Result<()> {
    let mut names = Vec::new();
    for parameter in parameters {
        let name = required_text("parameter.name", parameter.name.as_str())?;
        if names.contains(&name) {
            return Err(malformed_field(
                "parameter.name",
                format!("parameter {name:?} is already declared in this template"),
            ));
        }
        names.push(name);
        if let Some(description) = &parameter.description {
            required_text("parameter.description", description.as_str())?;
        }
        if let Some(default_value) = &parameter.default_value {
            required_text("parameter.default_value", default_value.as_str())?;
        }
    }
    Ok(())
}

fn missing_reference(field: &'static str, id: &Id) -> CoreError {
    malformed_field(
        field,
        format!("referenced identifier {id} is not registered"),
    )
}
