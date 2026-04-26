use crate::{CoreError, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

const CUSTOM_PREFIX: &str = "custom:";

/// Category of source material behind an observation or inference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SourceKind {
    /// Document source material.
    Document,
    /// Log source material.
    Log,
    /// API source material.
    Api,
    /// Human-provided source material.
    Human,
    /// AI-generated or AI-inferred source material.
    Ai,
    /// Code source material.
    Code,
    /// External source material outside the local system.
    External,
    /// Explicit extension category owned by a downstream crate or product.
    Custom(String),
}

impl SourceKind {
    /// Creates a custom source kind extension.
    pub fn custom(extension: impl Into<String>) -> Result<Self> {
        let raw = extension.into();
        let normalized = raw.trim().to_owned();

        if normalized.is_empty() {
            return Err(CoreError::invalid_source_kind(
                raw,
                "custom source kind extension must not be empty after trimming",
            ));
        }

        Ok(Self::Custom(normalized))
    }

    /// Returns true when this is a downstream-owned custom extension.
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Returns the stable serialized string for validated source kinds.
    ///
    /// Use [`Self::try_serialized_value`] when `Custom` may have been constructed directly.
    pub fn serialized_value(&self) -> String {
        self.try_serialized_value()
            .unwrap_or_else(|_| CUSTOM_PREFIX.to_owned())
    }

    /// Returns the stable serialized string after validating custom extensions.
    pub fn try_serialized_value(&self) -> Result<String> {
        match self {
            Self::Document => Ok("document".to_owned()),
            Self::Log => Ok("log".to_owned()),
            Self::Api => Ok("api".to_owned()),
            Self::Human => Ok("human".to_owned()),
            Self::Ai => Ok("ai".to_owned()),
            Self::Code => Ok("code".to_owned()),
            Self::External => Ok("external".to_owned()),
            Self::Custom(extension) => {
                let custom = Self::custom(extension.clone())?;
                let Self::Custom(normalized) = custom else {
                    unreachable!("SourceKind::custom always returns a custom source kind");
                };
                Ok(format!("{CUSTOM_PREFIX}{normalized}"))
            }
        }
    }
}

impl FromStr for SourceKind {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "document" => Ok(Self::Document),
            "log" => Ok(Self::Log),
            "api" => Ok(Self::Api),
            "human" => Ok(Self::Human),
            "ai" => Ok(Self::Ai),
            "code" => Ok(Self::Code),
            "external" => Ok(Self::External),
            custom if custom.starts_with(CUSTOM_PREFIX) => {
                Self::custom(&custom[CUSTOM_PREFIX.len()..])
            }
            unknown => Err(CoreError::invalid_source_kind(
                unknown,
                "expected document, log, api, human, ai, code, external, or custom:<extension>",
            )),
        }
    }
}

impl Serialize for SourceKind {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = self
            .try_serialized_value()
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&value)
    }
}

impl<'de> Deserialize<'de> for SourceKind {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for SourceKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.serialized_value())
    }
}

/// Portable reference to source material.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRef {
    /// Source category.
    pub kind: SourceKind,
    /// Optional stable URI for source material.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Optional human-readable source title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional stable text capture time, such as RFC 3339.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
    /// Optional identifier meaningful within the source system.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_local_id: Option<String>,
}

impl SourceRef {
    /// Creates a source reference with no optional metadata.
    pub fn new(kind: SourceKind) -> Self {
        Self {
            kind,
            uri: None,
            title: None,
            captured_at: None,
            source_local_id: None,
        }
    }
}
