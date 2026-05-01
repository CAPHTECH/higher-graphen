use super::common::{Description, LifecycleStatus, ObjectRef};
use super::validation::{require_non_empty, require_some};
use crate::text::normalize_required_text;
use crate::{Confidence, CoreError, Id, Provenance, Result};
use serde::{Deserialize, Serialize};

/// Direction for a valuation criterion.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CriterionDirection {
    /// Maximize this criterion.
    Maximize,
    /// Minimize this criterion.
    Minimize,
    /// Preserve this criterion.
    Preserve,
    /// Avoid this criterion.
    Avoid,
}

/// Criterion used in a valuation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValuationCriterion {
    /// Criterion identifier.
    pub criterion_id: String,
    /// Human-readable name.
    pub name: String,
    /// Optimization direction.
    pub direction: CriterionDirection,
    /// Optional weight.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    /// Whether the criterion is mandatory.
    pub required: bool,
}

impl ValuationCriterion {
    fn validate(&self) -> Result<()> {
        normalize_required_text("criteria.criterion_id", &self.criterion_id)?;
        normalize_required_text("criteria.name", &self.name)?;
        if let Some(weight) = self.weight {
            if !weight.is_finite() || weight < 0.0 {
                return Err(CoreError::malformed_field(
                    "criteria.weight",
                    "weight must be finite and non-negative",
                ));
            }
        }
        Ok(())
    }
}

/// Valuation ordering mode.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    /// Scalar score.
    ScalarScore,
    /// Lexicographic ordering.
    LexicographicOrder,
    /// Partial order.
    PartialOrder,
    /// Pareto frontier.
    ParetoFrontier,
    /// Threshold acceptance.
    ThresholdAcceptance,
    /// Qualitative ranking.
    QualitativeRanking,
}

/// Value attached to a valuation criterion.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CriterionValue {
    /// Criterion id.
    pub criterion_id: String,
    /// Stable value representation.
    pub value: ValuationValue,
    /// Evidence witness.
    pub evidence: Id,
}

impl CriterionValue {
    fn validate(&self) -> Result<()> {
        normalize_required_text("values.criterion_id", &self.criterion_id)?;
        Ok(())
    }
}

/// Primitive valuation value.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ValuationValue {
    /// String value.
    Text(String),
    /// Numeric value.
    Number(f64),
    /// Boolean value.
    Boolean(bool),
}

/// Trade-off recorded by a valuation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tradeoff {
    /// Gains introduced by the target.
    pub gains: String,
    /// Losses introduced by the target.
    pub losses: String,
    /// Affected invariants.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_invariants: Vec<Id>,
}

impl Tradeoff {
    fn validate(&self) -> Result<()> {
        normalize_required_text("tradeoffs.gains", &self.gains)?;
        normalize_required_text("tradeoffs.losses", &self.losses)?;
        Ok(())
    }
}

/// Value judgment under an explicit evaluation context.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Valuation {
    /// Valuation identifier.
    pub id: Id,
    /// Valued target.
    pub target: ObjectRef,
    /// Evaluation context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valuation_context: Option<Id>,
    /// Evaluation criteria.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criteria: Vec<ValuationCriterion>,
    /// Ordering mode.
    pub order_type: OrderType,
    /// Criterion values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<CriterionValue>,
    /// Recorded trade-offs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tradeoffs: Vec<Tradeoff>,
    /// Valuations that cannot be compared with this one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incomparable_with: Vec<Id>,
    /// Confidence in the valuation.
    pub confidence: Confidence,
    /// Valuation provenance.
    pub provenance: Provenance,
    /// Review lifecycle state.
    pub review_status: LifecycleStatus,
}

impl Valuation {
    /// Validates conditions required before using the valuation in a decision.
    pub fn validate_for_decision(&self) -> Result<()> {
        require_some("valuation_context", self.valuation_context.as_ref())?;
        require_non_empty("criteria", &self.criteria)?;
        require_non_empty("values", &self.values)?;
        for criterion in &self.criteria {
            criterion.validate()?;
        }
        for value in &self.values {
            value.validate()?;
        }
        for tradeoff in &self.tradeoffs {
            tradeoff.validate()?;
        }
        if !self.incomparable_with.is_empty()
            && matches!(
                self.order_type,
                OrderType::ScalarScore | OrderType::LexicographicOrder
            )
        {
            return Err(CoreError::malformed_field(
                "incomparable_with",
                "incomparable valuations must not be projected as a single ranking",
            ));
        }
        Ok(())
    }
}

/// Schema mapping kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaMappingKind {
    /// Rename mapping.
    Rename,
    /// Split mapping.
    Split,
    /// Merge mapping.
    Merge,
    /// Refinement mapping.
    Refinement,
    /// Abstraction mapping.
    Abstraction,
    /// Deprecation mapping.
    Deprecation,
    /// Semantic redefinition.
    SemanticRedefinition,
    /// Custom mapping.
    Custom,
}

/// Compatibility result for a schema morphism.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaCompatibility {
    /// Backward compatible mapping.
    BackwardCompatible,
    /// Forward compatible mapping.
    ForwardCompatible,
    /// Lossy mapping.
    Lossy,
    /// Incompatible mapping.
    Incompatible,
    /// Unknown compatibility.
    Unknown,
}

/// Single mapping inside a schema morphism.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaMapping {
    /// Source schema reference.
    pub source_ref: String,
    /// Target schema reference.
    pub target_ref: String,
    /// Mapping rule.
    pub mapping_rule: String,
    /// Invariant preservation claims.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preservation_claims: Vec<Id>,
    /// Loss claims.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loss_claims: Vec<Description>,
    /// Required review policies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_reviews: Vec<Id>,
}

impl SchemaMapping {
    fn validate(&self) -> Result<()> {
        normalize_required_text("mappings.source_ref", &self.source_ref)?;
        normalize_required_text("mappings.target_ref", &self.target_ref)?;
        normalize_required_text("mappings.mapping_rule", &self.mapping_rule)?;
        Ok(())
    }
}

/// Verification references for a schema morphism.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaVerification {
    /// Derivation checks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<Id>,
    /// Witness checks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub witnesses: Vec<Id>,
}

/// Morphism describing schema, ontology, or interpretation package evolution.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaMorphism {
    /// Schema morphism identifier.
    pub id: Id,
    /// Source schema.
    pub source_schema: Id,
    /// Target schema.
    pub target_schema: Id,
    /// Source interpretation package.
    pub source_interpretation_package: Id,
    /// Target interpretation package.
    pub target_interpretation_package: Id,
    /// Mapping kind.
    pub mapping_kind: SchemaMappingKind,
    /// Individual mappings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mappings: Vec<SchemaMapping>,
    /// Affected objects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_objects: Vec<ObjectRef>,
    /// Compatibility classification.
    pub compatibility: SchemaCompatibility,
    /// Verification checks.
    pub verification: SchemaVerification,
    /// Schema morphism provenance.
    pub provenance: Provenance,
    /// Review lifecycle state.
    pub review_status: LifecycleStatus,
}

impl SchemaMorphism {
    /// Validates conditions required before applying the schema morphism.
    pub fn validate_application(&self) -> Result<()> {
        require_non_empty("mappings", &self.mappings)?;
        for mapping in &self.mappings {
            mapping.validate()?;
        }
        if self.mapping_kind == SchemaMappingKind::SemanticRedefinition
            && self.review_status != LifecycleStatus::Accepted
        {
            return Err(CoreError::malformed_field(
                "review_status",
                "semantic redefinition requires explicit accepted review",
            ));
        }
        if matches!(
            self.mapping_kind,
            SchemaMappingKind::Split | SchemaMappingKind::Merge
        ) {
            require_non_empty("affected_objects", &self.affected_objects)?;
            if self
                .mappings
                .iter()
                .all(|mapping| mapping.loss_claims.is_empty())
            {
                return Err(CoreError::malformed_field(
                    "mappings.loss_claims",
                    "split or merge schema mapping requires loss claims",
                ));
            }
        }
        if self.compatibility == SchemaCompatibility::Incompatible {
            return Err(CoreError::malformed_field(
                "compatibility",
                "incompatible schema morphism cannot be applied as a migration",
            ));
        }
        if self.compatibility == SchemaCompatibility::Lossy
            && self
                .mappings
                .iter()
                .all(|mapping| mapping.loss_claims.is_empty())
        {
            return Err(CoreError::malformed_field(
                "mappings.loss_claims",
                "lossy schema morphism requires explicit loss claims",
            ));
        }
        Ok(())
    }
}
