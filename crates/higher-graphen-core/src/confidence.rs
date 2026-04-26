use crate::{CoreError, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Validated confidence score for extracted or inferred structure.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Confidence(f64);

impl Confidence {
    /// Minimum valid confidence score.
    pub const MIN: f64 = 0.0;
    /// Maximum valid confidence score.
    pub const MAX: f64 = 1.0;
    /// Valid zero confidence.
    pub const ZERO: Self = Self(Self::MIN);
    /// Valid full confidence.
    pub const ONE: Self = Self(Self::MAX);

    /// Creates a confidence score in the inclusive range `0.0..=1.0`.
    pub fn new(value: f64) -> Result<Self> {
        if !value.is_finite() {
            return Err(CoreError::invalid_confidence(
                value,
                "confidence must be finite",
            ));
        }

        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(CoreError::invalid_confidence(
                value,
                "confidence must be between 0.0 and 1.0 inclusive",
            ));
        }

        let normalized = if value == 0.0 { 0.0 } else { value };
        Ok(Self(normalized))
    }

    /// Returns true when the supplied value can be represented as a confidence score.
    pub fn is_valid_value(value: f64) -> bool {
        value.is_finite() && (Self::MIN..=Self::MAX).contains(&value)
    }

    /// Returns the validated numeric confidence score.
    pub fn value(self) -> f64 {
        self.0
    }

    /// Returns true for exactly zero confidence.
    pub fn is_zero(self) -> bool {
        self.0.to_bits() == Self::MIN.to_bits()
    }

    /// Returns true for exactly full confidence.
    pub fn is_certain(self) -> bool {
        self.0.to_bits() == Self::MAX.to_bits()
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
