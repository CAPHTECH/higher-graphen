//! Typestate review provenance for opt-in candidate-to-accepted promotion.
//!
//! Deserialization deliberately returns [`Reviewed<T, Candidate>`] even when
//! input claims `reviewStatus: "accepted"`; persisted data is re-entered as an
//! untrusted candidate that must be accepted again in-process.
//!
//! ```compile_fail
//! use higher_graphen_core::typed_provenance::{Accepted, Reviewed};
//!
//! // This must not compile -- no public constructor for Accepted state.
//! let _: Reviewed<String, Accepted> = Reviewed {
//!     value: "x".to_string(),
//!     provenance: todo!(),
//!     review: None,
//!     _state: std::marker::PhantomData,
//! };
//! ```

use crate::{Confidence, Id, Provenance, ReviewStatus, SourceKind, SourceRef};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::marker::PhantomData;

mod sealed {
    pub trait ReviewStateSealed {}
}

/// A sealed review-state marker that projects to the runtime [`ReviewStatus`].
pub trait ReviewState: sealed::ReviewStateSealed {
    /// Returns the runtime status represented by this typestate marker.
    fn status() -> ReviewStatus;
}

/// Marker for a value that is still a review candidate.
pub struct Candidate;

/// Marker for a value that has passed an explicit review morphism.
pub struct Accepted;

impl sealed::ReviewStateSealed for Candidate {}

impl sealed::ReviewStateSealed for Accepted {}

impl ReviewState for Candidate {
    fn status() -> ReviewStatus {
        ReviewStatus::Candidate
    }
}

impl ReviewState for Accepted {
    fn status() -> ReviewStatus {
        ReviewStatus::Accepted
    }
}

/// An explicit review act that authorizes candidate promotion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReviewMorphism {
    /// Identifier for the reviewer or review workflow that performed the act.
    pub reviewer_id: Id,
    /// Optional human-readable note describing the review decision.
    pub review_note: Option<String>,
}

/// A value carrying provenance and review status in its type.
pub struct Reviewed<T, S: ReviewState> {
    /// Wrapped payload value.
    value: T,
    /// Provenance attached to the reviewed payload.
    provenance: Provenance,
    /// Explicit review act, present only after promotion.
    review: Option<ReviewMorphism>,
    /// Typestate marker preventing construction without the legal transition.
    _state: PhantomData<S>,
}

impl<T> Reviewed<T, Candidate> {
    /// Creates a candidate reviewed value.
    pub fn candidate(value: T, provenance: Provenance) -> Reviewed<T, Candidate> {
        Reviewed {
            value,
            provenance,
            review: None,
            _state: PhantomData,
        }
    }

    /// Promotes a candidate to accepted by consuming it with an explicit review.
    pub fn accept(self, review: ReviewMorphism) -> Reviewed<T, Accepted> {
        Reviewed {
            value: self.value,
            provenance: self.provenance,
            review: Some(review),
            _state: PhantomData,
        }
    }
}

impl<T, S: ReviewState> Reviewed<T, S> {
    /// Returns the wrapped payload.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the provenance attached to the payload.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Returns the explicit review act, if one has occurred.
    pub fn review(&self) -> Option<&ReviewMorphism> {
        self.review.as_ref()
    }

    /// Returns the runtime review status represented by this typestate.
    pub fn review_status(&self) -> ReviewStatus {
        S::status()
    }
}

impl<T, S> Serialize for Reviewed<T, S>
where
    T: Serialize,
    S: ReviewState,
{
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        let mut state = serializer.serialize_struct("Reviewed", 3)?;
        state.serialize_field("payload", &self.value)?;
        state.serialize_field("reviewStatus", &S::status())?;
        state.serialize_field("review", &self.review)?;
        state.end()
    }
}

impl<'de, T> Deserialize<'de> for Reviewed<T, Candidate>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ReviewedCandidateWire<T> {
            payload: T,
        }

        let wire = ReviewedCandidateWire::<T>::deserialize(deserializer)?;
        Ok(Self {
            value: wire.payload,
            provenance: deserialized_candidate_provenance(),
            review: None,
            _state: PhantomData,
        })
    }
}

fn deserialized_candidate_provenance() -> Provenance {
    Provenance::new(SourceRef::new(SourceKind::External), Confidence::ZERO)
        .with_review_status(ReviewStatus::Candidate)
}

#[cfg(test)]
mod tests {
    use super::{Candidate, ReviewMorphism, Reviewed};
    use crate::{Confidence, Id, ParticipantRef, Provenance, ReviewStatus, SourceKind, SourceRef};

    fn id(value: &str) -> Id {
        Id::new(value).expect("test id should be valid")
    }

    fn provenance() -> Provenance {
        Provenance::new(SourceRef::new(SourceKind::Ai), Confidence::ONE)
            .with_review_status(ReviewStatus::Candidate)
    }

    fn review_morphism() -> ReviewMorphism {
        ReviewMorphism {
            reviewer_id: id("reviewer:typed-provenance"),
            review_note: Some("accepted in typed-provenance test".to_owned()),
        }
    }

    #[test]
    fn legal_candidate_accept_path_yields_accepted_status() {
        let accepted =
            Reviewed::candidate("hello".to_owned(), provenance()).accept(review_morphism());

        assert_eq!(accepted.review_status(), ReviewStatus::Accepted);
        assert!(accepted.review().is_some());
    }

    #[test]
    fn candidate_serde_roundtrips_as_candidate() {
        let candidate = Reviewed::candidate("hello".to_owned(), provenance());

        let serialized = serde_json::to_string(&candidate).expect("serialize candidate");
        let roundtrip: Reviewed<String, Candidate> =
            serde_json::from_str(&serialized).expect("deserialize candidate");

        assert_eq!(roundtrip.value(), "hello");
        assert_eq!(roundtrip.review_status(), ReviewStatus::Candidate);
        assert!(roundtrip.review().is_none());
    }

    #[test]
    fn accepted_json_deserializes_as_candidate() {
        let json = r#"{
            "payload": "hello",
            "reviewStatus": "accepted",
            "review": {
                "reviewer_id": "reviewer:persisted",
                "review_note": "persisted acceptance"
            }
        }"#;

        let candidate: Reviewed<String, Candidate> =
            serde_json::from_str(json).expect("deserialize as candidate");

        assert_eq!(candidate.value(), "hello");
        assert_eq!(candidate.review_status(), ReviewStatus::Candidate);
        assert!(candidate.review().is_none());
    }

    /// Adoption recipe: wrap the candidate payload with
    /// `Reviewed::candidate(payload, provenance)` at the boundary, then call
    /// `.accept(review_morphism)` only when an explicit review act is available.
    #[test]
    fn real_core_completion_candidate_participant_can_be_accepted() {
        let completion_candidate =
            ParticipantRef::CompletionCandidate(id("completion:typed-provenance"));

        let accepted =
            Reviewed::candidate(completion_candidate, provenance()).accept(review_morphism());

        assert_eq!(accepted.review_status(), ReviewStatus::Accepted);
    }
}
