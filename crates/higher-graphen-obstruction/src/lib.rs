//! Obstructions, counterexamples, engines, and explanations for HigherGraphen.

use higher_graphen_core::{CoreError, Id, Provenance, Result, Severity};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::str::FromStr;

const CUSTOM_OBSTRUCTION_PREFIX: &str = "custom:";

/// Type of structured failure recorded by an [`Obstruction`].
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ObstructionType {
    /// A checkable constraint did not hold.
    ConstraintUnsatisfied,
    /// An invariant that should be preserved was violated.
    InvariantViolation,
    /// Local structures could not be glued into a coherent global structure.
    FailedGluing,
    /// A morphism composition failed a compatibility or preservation check.
    FailedComposition,
    /// A required morphism is absent.
    MissingMorphism,
    /// A cell, invariant, or morphism is being interpreted in an incompatible context.
    ContextMismatch,
    /// A projection would lose structure required by the requested operation.
    ProjectionLoss,
    /// A required region, cell set, or context set is not covered.
    UncoveredRegion,
    /// Extension point for downstream obstruction categories.
    Custom(String),
}

impl ObstructionType {
    /// Creates a downstream-owned obstruction type extension.
    pub fn custom(extension: impl Into<String>) -> Result<Self> {
        Ok(Self::Custom(normalized_required_text(
            "obstruction_type",
            extension,
        )?))
    }

    /// Returns true when this is a downstream-owned custom extension.
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Returns the stable serialized string for this obstruction type.
    pub fn serialized_value(&self) -> Result<String> {
        match self {
            Self::ConstraintUnsatisfied => Ok("constraint_unsatisfied".to_owned()),
            Self::InvariantViolation => Ok("invariant_violation".to_owned()),
            Self::FailedGluing => Ok("failed_gluing".to_owned()),
            Self::FailedComposition => Ok("failed_composition".to_owned()),
            Self::MissingMorphism => Ok("missing_morphism".to_owned()),
            Self::ContextMismatch => Ok("context_mismatch".to_owned()),
            Self::ProjectionLoss => Ok("projection_loss".to_owned()),
            Self::UncoveredRegion => Ok("uncovered_region".to_owned()),
            Self::Custom(extension) => {
                let extension = normalized_required_text("obstruction_type", extension)?;
                Ok(format!("{CUSTOM_OBSTRUCTION_PREFIX}{extension}"))
            }
        }
    }
}

impl FromStr for ObstructionType {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "constraint_unsatisfied" => Ok(Self::ConstraintUnsatisfied),
            "invariant_violation" => Ok(Self::InvariantViolation),
            "failed_gluing" => Ok(Self::FailedGluing),
            "failed_composition" => Ok(Self::FailedComposition),
            "missing_morphism" => Ok(Self::MissingMorphism),
            "context_mismatch" => Ok(Self::ContextMismatch),
            "projection_loss" => Ok(Self::ProjectionLoss),
            "uncovered_region" => Ok(Self::UncoveredRegion),
            custom if custom.starts_with(CUSTOM_OBSTRUCTION_PREFIX) => {
                Self::custom(&custom[CUSTOM_OBSTRUCTION_PREFIX.len()..])
            }
            unknown => Err(CoreError::ParseFailure {
                target: "ObstructionType".to_owned(),
                value: unknown.to_owned(),
                reason: "expected a known obstruction type or custom:<extension>".to_owned(),
            }),
        }
    }
}

impl Serialize for ObstructionType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.serialized_value().map_err(serde::ser::Error::custom)?)
    }
}

impl<'de> Deserialize<'de> for ObstructionType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

/// Plain, projection-neutral explanation of why an obstruction exists.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObstructionExplanation {
    /// Short human-readable summary.
    pub summary: String,
    /// Optional additional detail for direct inspection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ObstructionExplanation {
    /// Creates an explanation with a required non-empty summary.
    pub fn new(summary: impl Into<String>) -> Result<Self> {
        Ok(Self {
            summary: normalized_required_text("explanation.summary", summary)?,
            details: None,
        })
    }

    /// Returns this explanation with optional additional detail.
    pub fn with_details(mut self, details: impl Into<String>) -> Result<Self> {
        self.details = Some(normalized_required_text("explanation.details", details)?);
        Ok(self)
    }
}

/// Structured hint describing what must be resolved before an obstruction clears.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredResolution {
    /// Short human-readable resolution requirement.
    pub summary: String,
    /// Cells that should be changed, supplied, or reviewed to resolve the obstruction.
    #[serde(default)]
    pub target_cell_ids: Vec<Id>,
    /// Contexts that should be changed, supplied, or reviewed to resolve the obstruction.
    #[serde(default)]
    pub target_context_ids: Vec<Id>,
    /// Morphisms that should be changed, supplied, or reviewed to resolve the obstruction.
    #[serde(default)]
    pub target_morphism_ids: Vec<Id>,
}

impl RequiredResolution {
    /// Creates a resolution hint with no structural targets yet attached.
    pub fn new(summary: impl Into<String>) -> Result<Self> {
        Ok(Self {
            summary: normalized_required_text("required_resolution.summary", summary)?,
            target_cell_ids: Vec::new(),
            target_context_ids: Vec::new(),
            target_morphism_ids: Vec::new(),
        })
    }

    /// Adds a cell target to this resolution hint.
    pub fn with_target_cell(mut self, cell_id: Id) -> Self {
        push_unique(&mut self.target_cell_ids, cell_id);
        self
    }

    /// Adds a context target to this resolution hint.
    pub fn with_target_context(mut self, context_id: Id) -> Self {
        push_unique(&mut self.target_context_ids, context_id);
        self
    }

    /// Adds a morphism target to this resolution hint.
    pub fn with_target_morphism(mut self, morphism_id: Id) -> Self {
        push_unique(&mut self.target_morphism_ids, morphism_id);
        self
    }
}

/// Morphism related to an obstruction, with an optional role and explanation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RelatedMorphism {
    /// Related morphism identifier.
    pub morphism_id: Id,
    /// Optional stable role label such as `failed_member`, `required`, or `witness`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Optional note explaining how the morphism participates in the obstruction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

impl RelatedMorphism {
    /// Creates a related morphism reference with no role metadata.
    pub fn new(morphism_id: Id) -> Self {
        Self {
            morphism_id,
            role: None,
            explanation: None,
        }
    }

    /// Returns this relation with a stable non-empty role label.
    pub fn with_role(mut self, role: impl Into<String>) -> Result<Self> {
        self.role = Some(normalized_required_text("related_morphism.role", role)?);
        Ok(self)
    }

    /// Returns this relation with a non-empty explanation.
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Result<Self> {
        self.explanation = Some(normalized_required_text(
            "related_morphism.explanation",
            explanation,
        )?);
        Ok(self)
    }
}

/// Concrete witness showing that an obstruction is real.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Counterexample {
    /// Human-readable witness description.
    pub description: String,
    /// Stable textual assignments used by the witness.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub assignments: BTreeMap<String, String>,
    /// Cell path or witness cells involved in the counterexample.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path_cell_ids: Vec<Id>,
    /// Contexts involved in the counterexample.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
}

impl Counterexample {
    /// Creates a counterexample with a required non-empty description.
    pub fn new(description: impl Into<String>) -> Result<Self> {
        Ok(Self {
            description: normalized_required_text("counterexample.description", description)?,
            assignments: BTreeMap::new(),
            path_cell_ids: Vec::new(),
            context_ids: Vec::new(),
        })
    }

    /// Adds a stable textual assignment to the counterexample.
    pub fn with_assignment(
        mut self,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self> {
        self.assignments.insert(
            normalized_required_text("counterexample.assignments.name", name)?,
            normalized_required_text("counterexample.assignments.value", value)?,
        );
        Ok(self)
    }

    /// Adds a cell to the witness path.
    pub fn with_path_cell(mut self, cell_id: Id) -> Self {
        push_unique(&mut self.path_cell_ids, cell_id);
        self
    }

    /// Adds a context to the witness.
    pub fn with_context(mut self, context_id: Id) -> Self {
        push_unique(&mut self.context_ids, context_id);
        self
    }
}

/// Structured failure record owned by the obstruction package.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Obstruction {
    /// Obstruction identifier.
    pub id: Id,
    /// Owning space identifier.
    pub space_id: Id,
    /// Type of structured failure.
    pub obstruction_type: ObstructionType,
    /// Cells where the obstruction occurs.
    #[serde(default)]
    pub location_cell_ids: Vec<Id>,
    /// Contexts where the obstruction occurs.
    #[serde(default)]
    pub location_context_ids: Vec<Id>,
    /// Morphisms related to the obstruction.
    #[serde(default)]
    pub related_morphisms: Vec<RelatedMorphism>,
    /// Projection-neutral explanation of the obstruction.
    pub explanation: ObstructionExplanation,
    /// Optional concrete witness for the obstruction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counterexample: Option<Counterexample>,
    /// Impact classification.
    pub severity: Severity,
    /// Optional requirement that must be satisfied before the obstruction clears.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_resolution: Option<RequiredResolution>,
    /// Source and review metadata.
    pub provenance: Provenance,
}

impl Obstruction {
    /// Creates an obstruction with empty location and relation sets.
    pub fn new(
        id: Id,
        space_id: Id,
        obstruction_type: ObstructionType,
        explanation: ObstructionExplanation,
        severity: Severity,
        provenance: Provenance,
    ) -> Self {
        Self {
            id,
            space_id,
            obstruction_type,
            location_cell_ids: Vec::new(),
            location_context_ids: Vec::new(),
            related_morphisms: Vec::new(),
            explanation,
            counterexample: None,
            severity,
            required_resolution: None,
            provenance,
        }
    }

    /// Adds a cell location to this obstruction.
    pub fn with_location_cell(mut self, cell_id: Id) -> Self {
        push_unique(&mut self.location_cell_ids, cell_id);
        self
    }

    /// Adds a context location to this obstruction.
    pub fn with_location_context(mut self, context_id: Id) -> Self {
        push_unique(&mut self.location_context_ids, context_id);
        self
    }

    /// Adds a related morphism to this obstruction.
    pub fn with_related_morphism(mut self, related_morphism: RelatedMorphism) -> Self {
        push_unique(&mut self.related_morphisms, related_morphism);
        self
    }

    /// Attaches a concrete counterexample.
    pub fn with_counterexample(mut self, counterexample: Counterexample) -> Self {
        self.counterexample = Some(counterexample);
        self
    }

    /// Attaches a required resolution hint.
    pub fn with_required_resolution(mut self, required_resolution: RequiredResolution) -> Self {
        self.required_resolution = Some(required_resolution);
        self
    }

    /// Returns true when the obstruction has a concrete counterexample.
    pub fn has_counterexample(&self) -> bool {
        self.counterexample.is_some()
    }

    /// Returns true when the obstruction carries a resolution requirement.
    pub fn requires_resolution(&self) -> bool {
        self.required_resolution.is_some()
    }
}

fn normalized_required_text(field: &'static str, value: impl Into<String>) -> Result<String> {
    let raw = value.into();
    let normalized = raw.trim().to_owned();

    if normalized.is_empty() {
        return Err(CoreError::MalformedField {
            field: field.to_owned(),
            reason: "value must not be empty after trimming".to_owned(),
        });
    }

    Ok(normalized)
}

fn push_unique<T: Eq>(items: &mut Vec<T>, item: T) {
    if !items.contains(&item) {
        items.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Counterexample, Obstruction, ObstructionExplanation, ObstructionType, RelatedMorphism,
        RequiredResolution,
    };
    use higher_graphen_core::{Confidence, Id, Provenance, Severity, SourceKind, SourceRef};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::str::FromStr;

    fn assert_serde_contract<T>()
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
    }

    fn id(value: &str) -> Id {
        Id::new(value).expect("valid id")
    }

    fn provenance() -> Provenance {
        Provenance::new(
            SourceRef::new(SourceKind::Code),
            Confidence::new(0.9).expect("valid confidence"),
        )
    }

    #[test]
    fn obstruction_type_uses_stable_string_values() {
        assert_eq!(
            ObstructionType::from_str("failed_composition").expect("parse type"),
            ObstructionType::FailedComposition
        );
        assert_eq!(
            ObstructionType::custom("domain_specific")
                .expect("custom type")
                .serialized_value()
                .expect("serialized value"),
            "custom:domain_specific"
        );
        assert!(ObstructionType::custom(" ").is_err());
        assert!(ObstructionType::from_str("unknown").is_err());
    }

    #[test]
    fn obstruction_records_locations_morphisms_resolution_and_counterexample() {
        let counterexample = Counterexample::new("cell b violates invariant a")
            .expect("counterexample")
            .with_assignment("cell", "cell/b")
            .expect("assignment")
            .with_path_cell(id("cell/a"))
            .with_path_cell(id("cell/b"))
            .with_context(id("context/local"));
        let related_morphism = RelatedMorphism::new(id("morphism/m1"))
            .with_role("failed_member")
            .expect("role")
            .with_explanation("composition loses required cell")
            .expect("explanation");
        let resolution = RequiredResolution::new("supply a preserving morphism")
            .expect("resolution")
            .with_target_cell(id("cell/b"))
            .with_target_context(id("context/local"))
            .with_target_morphism(id("morphism/m1"));
        let obstruction = Obstruction::new(
            id("obstruction/o1"),
            id("space/main"),
            ObstructionType::InvariantViolation,
            ObstructionExplanation::new("required invariant does not hold")
                .expect("explanation")
                .with_details("the local witness cannot be extended globally")
                .expect("details"),
            Severity::High,
            provenance(),
        )
        .with_location_cell(id("cell/b"))
        .with_location_cell(id("cell/b"))
        .with_location_context(id("context/local"))
        .with_location_context(id("context/local"))
        .with_related_morphism(related_morphism)
        .with_counterexample(counterexample)
        .with_required_resolution(resolution);

        assert!(obstruction.has_counterexample());
        assert!(obstruction.requires_resolution());
        assert_eq!(obstruction.location_cell_ids, vec![id("cell/b")]);
        assert_eq!(obstruction.location_context_ids, vec![id("context/local")]);
        assert_eq!(
            obstruction.related_morphisms[0].morphism_id,
            id("morphism/m1")
        );
        assert_eq!(obstruction.severity, Severity::High);
    }

    #[test]
    fn serde_defaults_empty_obstruction_collections() {
        let value = json!({
            "id": "obstruction/o1",
            "space_id": "space/main",
            "obstruction_type": "missing_morphism",
            "explanation": {
                "summary": "required morphism is absent"
            },
            "severity": "high",
            "provenance": provenance()
        });

        let obstruction: Obstruction = serde_json::from_value(value).expect("obstruction");

        assert!(obstruction.location_cell_ids.is_empty());
        assert!(obstruction.location_context_ids.is_empty());
        assert!(obstruction.related_morphisms.is_empty());
        assert!(obstruction.counterexample.is_none());
        assert!(obstruction.required_resolution.is_none());
    }

    #[test]
    fn builders_deduplicate_set_like_targets_and_locations() {
        let resolution = RequiredResolution::new("review duplicate cell")
            .expect("resolution")
            .with_target_cell(id("cell/a"))
            .with_target_cell(id("cell/a"))
            .with_target_context(id("context/a"))
            .with_target_context(id("context/a"))
            .with_target_morphism(id("morphism/a"))
            .with_target_morphism(id("morphism/a"));
        let counterexample = Counterexample::new("duplicate witness ids")
            .expect("counterexample")
            .with_path_cell(id("cell/a"))
            .with_path_cell(id("cell/a"))
            .with_context(id("context/a"))
            .with_context(id("context/a"));

        assert_eq!(resolution.target_cell_ids, vec![id("cell/a")]);
        assert_eq!(resolution.target_context_ids, vec![id("context/a")]);
        assert_eq!(resolution.target_morphism_ids, vec![id("morphism/a")]);
        assert_eq!(counterexample.path_cell_ids, vec![id("cell/a")]);
        assert_eq!(counterexample.context_ids, vec![id("context/a")]);
    }

    #[test]
    fn required_human_text_is_validated_by_constructors() {
        assert!(ObstructionExplanation::new(" ").is_err());
        assert!(Counterexample::new("").is_err());
        assert!(RequiredResolution::new("\n").is_err());
        assert!(RelatedMorphism::new(id("morphism/m1"))
            .with_role(" ")
            .is_err());
    }

    #[test]
    fn public_types_implement_serde_contracts() {
        assert_serde_contract::<ObstructionType>();
        assert_serde_contract::<ObstructionExplanation>();
        assert_serde_contract::<RequiredResolution>();
        assert_serde_contract::<RelatedMorphism>();
        assert_serde_contract::<Counterexample>();
        assert_serde_contract::<Obstruction>();
    }
}
