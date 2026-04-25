use crate::{CoreError, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Opaque, stable identifier for HigherGraphen structures.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(String);

impl Id {
    /// Creates an identifier after trimming surrounding whitespace.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let raw = value.into();
        let normalized = raw.trim().to_owned();

        if normalized.is_empty() {
            return Err(CoreError::invalid_id(
                raw,
                "identifier must not be empty after trimming",
            ));
        }

        Ok(Self(normalized))
    }

    /// Returns the stable identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts the identifier into its stable string representation.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<String> for Id {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Id {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl FromStr for Id {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
