//! Domain interpretation packages, templates, mappings, projections, and lift adapters for HigherGraphen.

pub mod architecture;

mod package;

pub use package::InterpretationPackage;

use higher_graphen_core::{CoreError, Id, Provenance, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Generic string metadata carried by interpretation definitions.
///
/// Product packages can record product identity or vocabulary hints here
/// without adding product-specific dependencies to this crate.
pub type Metadata = BTreeMap<String, String>;

/// Product-neutral target category for a domain type mapping.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpretationTargetKind {
    /// The domain type maps to a HigherGraphen cell.
    Cell,
    /// The domain type maps to a HigherGraphen incidence.
    Incidence,
    /// The domain type maps to a HigherGraphen complex.
    Complex,
    /// The domain type maps to a HigherGraphen morphism.
    Morphism,
    /// The domain type maps to an invariant definition.
    Invariant,
    /// The domain type maps to a constraint definition.
    Constraint,
    /// The domain type maps to an obstruction definition.
    Obstruction,
    /// The domain type maps to a completion candidate definition.
    CompletionCandidate,
    /// The domain type maps to a projection definition.
    Projection,
    /// The domain type maps to a downstream-owned category.
    Custom(String),
}

impl InterpretationTargetKind {
    /// Creates a custom target kind with a non-empty extension name.
    pub fn custom(extension: impl Into<String>) -> Result<Self> {
        Ok(Self::Custom(required_text("target_kind", extension)?))
    }
}

/// Parameter accepted by an invariant or projection template.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateParameter {
    /// Stable parameter name.
    pub name: String,
    /// Whether callers must supply a value for this parameter.
    pub required: bool,
    /// Optional human-readable parameter description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional default value represented as transport-neutral text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

impl TemplateParameter {
    /// Creates a required template parameter.
    pub fn required(name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            name: required_text("parameter.name", name)?,
            required: true,
            description: None,
            default_value: None,
        })
    }

    /// Creates an optional template parameter.
    pub fn optional(name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            required: false,
            ..Self::required(name)?
        })
    }

    /// Returns this parameter with a non-empty description.
    pub fn with_description(mut self, description: impl Into<String>) -> Result<Self> {
        self.description = Some(required_text("parameter.description", description)?);
        Ok(self)
    }

    /// Returns this parameter with a non-empty default value.
    pub fn with_default_value(mut self, default_value: impl Into<String>) -> Result<Self> {
        self.default_value = Some(required_text("parameter.default_value", default_value)?);
        Ok(self)
    }
}

/// Mapping from a domain type name to a HigherGraphen target category and type name.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TypeMapping {
    /// Mapping identifier.
    pub id: Id,
    /// Domain-owned source type name.
    pub source_type: String,
    /// HigherGraphen target category.
    pub target_kind: InterpretationTargetKind,
    /// Target type name meaningful to the receiving HigherGraphen structure.
    pub target_type: String,
    /// Optional human-readable explanation of the mapping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Generic string metadata for product or tool hints.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: Metadata,
    /// Optional source and review metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl TypeMapping {
    /// Creates a domain type mapping with validated source and target names.
    pub fn new(
        id: Id,
        source_type: impl Into<String>,
        target_kind: InterpretationTargetKind,
        target_type: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            source_type: required_text("source_type", source_type)?,
            target_kind,
            target_type: required_text("target_type", target_type)?,
            description: None,
            metadata: Metadata::new(),
            provenance: None,
        })
    }

    /// Returns this mapping with a non-empty description.
    pub fn with_description(mut self, description: impl Into<String>) -> Result<Self> {
        self.description = Some(required_text("description", description)?);
        Ok(self)
    }

    /// Returns this mapping with one generic metadata entry.
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self> {
        insert_metadata(&mut self.metadata, key, value)?;
        Ok(self)
    }

    /// Returns this mapping with source and review metadata.
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Returns a metadata value by key after trimming the lookup key.
    #[must_use]
    pub fn metadata_value(&self, key: &str) -> Option<&str> {
        metadata_value(&self.metadata, key)
    }
}

/// Mapping from a domain relation or transformation type to a HigherGraphen morphism type label.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MorphismTypeMapping {
    /// Mapping identifier.
    pub id: Id,
    /// Domain-owned morphism or relation type name.
    pub source_type: String,
    /// Product-neutral HigherGraphen morphism type label.
    pub morphism_type: String,
    /// Source type mappings this morphism may start from.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_type_mapping_ids: Vec<Id>,
    /// Target type mappings this morphism may end at.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_type_mapping_ids: Vec<Id>,
    /// Invariant templates this morphism type is expected to preserve.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preserved_invariant_template_ids: Vec<Id>,
    /// Optional human-readable explanation of the mapping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Generic string metadata for product or tool hints.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: Metadata,
    /// Optional source and review metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl MorphismTypeMapping {
    /// Creates a morphism type mapping with validated labels.
    pub fn new(
        id: Id,
        source_type: impl Into<String>,
        morphism_type: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            source_type: required_text("source_type", source_type)?,
            morphism_type: required_text("morphism_type", morphism_type)?,
            source_type_mapping_ids: Vec::new(),
            target_type_mapping_ids: Vec::new(),
            preserved_invariant_template_ids: Vec::new(),
            description: None,
            metadata: Metadata::new(),
            provenance: None,
        })
    }

    /// Adds a source type mapping reference.
    pub fn with_source_type_mapping(mut self, type_mapping_id: Id) -> Self {
        push_unique(&mut self.source_type_mapping_ids, type_mapping_id);
        self
    }

    /// Adds a target type mapping reference.
    pub fn with_target_type_mapping(mut self, type_mapping_id: Id) -> Self {
        push_unique(&mut self.target_type_mapping_ids, type_mapping_id);
        self
    }

    /// Adds an invariant template this morphism type is expected to preserve.
    pub fn with_preserved_invariant_template(mut self, invariant_template_id: Id) -> Self {
        push_unique(
            &mut self.preserved_invariant_template_ids,
            invariant_template_id,
        );
        self
    }

    /// Returns this mapping with a non-empty description.
    pub fn with_description(mut self, description: impl Into<String>) -> Result<Self> {
        self.description = Some(required_text("description", description)?);
        Ok(self)
    }

    /// Returns this mapping with one generic metadata entry.
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self> {
        insert_metadata(&mut self.metadata, key, value)?;
        Ok(self)
    }

    /// Returns this mapping with source and review metadata.
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Returns a metadata value by key after trimming the lookup key.
    #[must_use]
    pub fn metadata_value(&self, key: &str) -> Option<&str> {
        metadata_value(&self.metadata, key)
    }
}

/// Reusable invariant definition template owned by an interpretation package.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InvariantTemplate {
    /// Template identifier.
    pub id: Id,
    /// Human-readable template name.
    pub name: String,
    /// Product-neutral statement of the invariant.
    pub statement: String,
    /// Template parameters required to instantiate the invariant.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<TemplateParameter>,
    /// Type mappings to which this invariant template applies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applies_to_type_mapping_ids: Vec<Id>,
    /// Generic string metadata for product or tool hints.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: Metadata,
    /// Optional source and review metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl InvariantTemplate {
    /// Creates an invariant template with validated text.
    pub fn new(id: Id, name: impl Into<String>, statement: impl Into<String>) -> Result<Self> {
        Ok(Self {
            id,
            name: required_text("name", name)?,
            statement: required_text("statement", statement)?,
            parameters: Vec::new(),
            applies_to_type_mapping_ids: Vec::new(),
            metadata: Metadata::new(),
            provenance: None,
        })
    }

    /// Adds a template parameter.
    pub fn with_parameter(mut self, parameter: TemplateParameter) -> Self {
        self.parameters.push(parameter);
        self
    }

    /// Adds a type mapping reference this template applies to.
    pub fn with_type_mapping(mut self, type_mapping_id: Id) -> Self {
        push_unique(&mut self.applies_to_type_mapping_ids, type_mapping_id);
        self
    }

    /// Returns this template with one generic metadata entry.
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self> {
        insert_metadata(&mut self.metadata, key, value)?;
        Ok(self)
    }

    /// Returns this template with source and review metadata.
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Returns a metadata value by key after trimming the lookup key.
    #[must_use]
    pub fn metadata_value(&self, key: &str) -> Option<&str> {
        metadata_value(&self.metadata, key)
    }

    /// Returns a parameter by stable parameter name.
    #[must_use]
    pub fn parameter(&self, name: &str) -> Option<&TemplateParameter> {
        let normalized = name.trim();
        self.parameters
            .iter()
            .find(|parameter| parameter.name == normalized)
    }
}

/// Reusable projection definition template owned by an interpretation package.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionTemplate {
    /// Template identifier.
    pub id: Id,
    /// Human-readable template name.
    pub name: String,
    /// Target audience label, represented without depending on projection crates.
    pub audience: String,
    /// Projection purpose label, represented without depending on projection crates.
    pub purpose: String,
    /// Expected output shape label, such as `text`, `sections`, or a custom name.
    pub output_shape: String,
    /// Template parameters required to instantiate the projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<TemplateParameter>,
    /// Type mappings selected as projection sources.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_type_mapping_ids: Vec<Id>,
    /// Invariant templates represented or summarized by this projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariant_template_ids: Vec<Id>,
    /// Generic string metadata for product or tool hints.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: Metadata,
    /// Optional source and review metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl ProjectionTemplate {
    /// Creates a projection template with validated text labels.
    pub fn new(
        id: Id,
        name: impl Into<String>,
        audience: impl Into<String>,
        purpose: impl Into<String>,
        output_shape: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            id,
            name: required_text("name", name)?,
            audience: required_text("audience", audience)?,
            purpose: required_text("purpose", purpose)?,
            output_shape: required_text("output_shape", output_shape)?,
            parameters: Vec::new(),
            source_type_mapping_ids: Vec::new(),
            invariant_template_ids: Vec::new(),
            metadata: Metadata::new(),
            provenance: None,
        })
    }

    /// Adds a template parameter.
    pub fn with_parameter(mut self, parameter: TemplateParameter) -> Self {
        self.parameters.push(parameter);
        self
    }

    /// Adds a source type mapping reference.
    pub fn with_source_type_mapping(mut self, type_mapping_id: Id) -> Self {
        push_unique(&mut self.source_type_mapping_ids, type_mapping_id);
        self
    }

    /// Adds an invariant template reference represented by this projection.
    pub fn with_invariant_template(mut self, invariant_template_id: Id) -> Self {
        push_unique(&mut self.invariant_template_ids, invariant_template_id);
        self
    }

    /// Returns this template with one generic metadata entry.
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self> {
        insert_metadata(&mut self.metadata, key, value)?;
        Ok(self)
    }

    /// Returns this template with source and review metadata.
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Returns a metadata value by key after trimming the lookup key.
    #[must_use]
    pub fn metadata_value(&self, key: &str) -> Option<&str> {
        metadata_value(&self.metadata, key)
    }
}

/// Definition for an adapter that can lift source input into interpretation-owned structures.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LiftAdapterDefinition {
    /// Adapter identifier.
    pub id: Id,
    /// Human-readable adapter name.
    pub name: String,
    /// Source input kind, such as a document family, schema format, or API shape.
    pub input_kind: String,
    /// Optional target output kind requested from the adapter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_kind: Option<String>,
    /// Type mappings supported by this adapter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_type_mapping_ids: Vec<Id>,
    /// Morphism type mappings supported by this adapter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_morphism_type_mapping_ids: Vec<Id>,
    /// Generic string metadata for product or tool hints.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: Metadata,
    /// Optional source and review metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

impl LiftAdapterDefinition {
    /// Creates a lift adapter definition with validated text.
    pub fn new(id: Id, name: impl Into<String>, input_kind: impl Into<String>) -> Result<Self> {
        Ok(Self {
            id,
            name: required_text("name", name)?,
            input_kind: required_text("input_kind", input_kind)?,
            output_kind: None,
            supported_type_mapping_ids: Vec::new(),
            supported_morphism_type_mapping_ids: Vec::new(),
            metadata: Metadata::new(),
            provenance: None,
        })
    }

    /// Returns this adapter with a non-empty output kind.
    pub fn with_output_kind(mut self, output_kind: impl Into<String>) -> Result<Self> {
        self.output_kind = Some(required_text("output_kind", output_kind)?);
        Ok(self)
    }

    /// Adds a supported type mapping reference.
    pub fn with_supported_type_mapping(mut self, type_mapping_id: Id) -> Self {
        push_unique(&mut self.supported_type_mapping_ids, type_mapping_id);
        self
    }

    /// Adds a supported morphism type mapping reference.
    pub fn with_supported_morphism_type_mapping(mut self, morphism_type_mapping_id: Id) -> Self {
        push_unique(
            &mut self.supported_morphism_type_mapping_ids,
            morphism_type_mapping_id,
        );
        self
    }

    /// Returns this adapter with one generic metadata entry.
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self> {
        insert_metadata(&mut self.metadata, key, value)?;
        Ok(self)
    }

    /// Returns this adapter with source and review metadata.
    pub fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Returns a metadata value by key after trimming the lookup key.
    #[must_use]
    pub fn metadata_value(&self, key: &str) -> Option<&str> {
        metadata_value(&self.metadata, key)
    }
}

fn insert_metadata(
    metadata: &mut Metadata,
    key: impl Into<String>,
    value: impl Into<String>,
) -> Result<()> {
    metadata.insert(
        required_text("metadata.key", key)?,
        required_text("metadata.value", value)?,
    );
    Ok(())
}

fn metadata_value<'a>(metadata: &'a Metadata, key: &str) -> Option<&'a str> {
    metadata.get(key.trim()).map(String::as_str)
}

fn required_text(field: &'static str, value: impl Into<String>) -> Result<String> {
    let raw = value.into();
    let normalized = raw.trim().to_owned();

    if normalized.is_empty() {
        return Err(malformed_field(
            field,
            "value must not be empty after trimming",
        ));
    }

    Ok(normalized)
}

fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn malformed_field(field: impl Into<String>, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
