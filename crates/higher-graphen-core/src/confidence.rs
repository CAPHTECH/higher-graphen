use crate::{CoreError, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Validated confidence score for extracted or inferred structure.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Confidence(f64);

impl Confidence {
    /// Creates a confidence score in the inclusive range `0.0..=1.0`.
    pub fn new(value: f64) -> Result<Self> {
        if !value.is_finite() {
            return Err(CoreError::invalid_confidence(
                value,
                "confidence must be finite",
            ));
        }

        if !(0.0..=1.0).contains(&value) {
            return Err(CoreError::invalid_confidence(
                value,
                "confidence must be between 0.0 and 1.0 inclusive",
            ));
        }

        Ok(Self(value))
    }

    /// Returns the validated numeric confidence score.
    pub fn value(self) -> f64 {
        self.0
    }
}

impl Serialize for Confidence {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for Confidence {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<f64> for Confidence {
    type Error = CoreError;

    fn try_from(value: f64) -> Result<Self> {
        Self::new(value)
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
