use crate::{Confidence, CoreError, Id, Provenance, ReviewStatus, Severity, SourceKind, SourceRef};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn id_roundtrips_as_string_and_remains_orderable() {
    let id = Id::new("  structure-001  ").expect("valid id");

    assert_eq!(id.as_str(), "structure-001");
    assert_eq!(
        serde_json::to_string(&id).expect("serialize id"),
        "\"structure-001\""
    );

    let roundtrip: Id = serde_json::from_str("\"structure-001\"").expect("deserialize id");
    assert_eq!(roundtrip, id);
    assert!(Id::new("a").expect("valid id") < Id::new("b").expect("valid id"));
}

#[test]
fn id_can_be_used_as_a_downstream_json_map_key() {
    let mut keyed = BTreeMap::new();
    keyed.insert(Id::new("structure/a").expect("valid id"), 1_u8);

    let json = serde_json::to_string(&keyed).expect("serialize keyed map");
    assert_eq!(json, r#"{"structure/a":1}"#);

    let roundtrip: BTreeMap<Id, u8> = serde_json::from_str(&json).expect("deserialize keyed map");
    assert_eq!(roundtrip[&Id::new("structure/a").expect("valid id")], 1);
}

#[test]
fn validation_failures_return_structured_core_errors() {
    assert_eq!(Id::new("   ").expect_err("empty id").code(), "invalid_id");
    assert_eq!(
        Id::new("structure\n001")
            .expect_err("control character id")
            .code(),
        "invalid_id"
    );
    assert!(!Id::is_valid_value("structure\t001"));

    for value in [f64::NAN, f64::INFINITY, -0.01, 1.01] {
        assert_eq!(
            Confidence::new(value)
                .expect_err("invalid confidence")
                .code(),
            "invalid_confidence"
        );
    }

    let error = serde_json::from_str::<Confidence>("1.01").expect_err("invalid confidence json");
    assert!(error.to_string().contains("invalid_confidence"));
}

#[test]
fn confidence_roundtrips_and_rejects_invalid_deserialized_values() {
    let confidence = Confidence::new(0.42).expect("valid confidence");

    let json = serde_json::to_string(&confidence).expect("serialize confidence");
    assert_eq!(json, "0.42");

    let roundtrip: Confidence = serde_json::from_str(&json).expect("deserialize confidence");
    assert!((roundtrip.value() - 0.42).abs() < f64::EPSILON);
    assert!(serde_json::from_str::<Confidence>("-0.1").is_err());

    let zero = Confidence::new(-0.0).expect("negative zero is in range");
    assert_eq!(zero.value().to_bits(), 0.0_f64.to_bits());
    assert_eq!(serde_json::to_string(&zero).expect("serialize zero"), "0.0");
    assert!(Confidence::ZERO.is_zero());
    assert!(Confidence::ONE.is_certain());
    assert!(Confidence::is_valid_value(1.0));
    assert!(!Confidence::is_valid_value(f64::NAN));
}

#[test]
fn source_kind_serializes_canonical_values_and_custom_extensions() {
    let cases = [
        (SourceKind::Document, "\"document\""),
        (SourceKind::Log, "\"log\""),
        (SourceKind::Api, "\"api\""),
        (SourceKind::Human, "\"human\""),
        (SourceKind::Ai, "\"ai\""),
        (SourceKind::Code, "\"code\""),
        (SourceKind::External, "\"external\""),
    ];

    for (kind, expected_json) in cases {
        assert_eq!(
            serde_json::to_string(&kind).expect("serialize kind"),
            expected_json
        );
        assert_eq!(
            serde_json::from_str::<SourceKind>(expected_json).expect("deserialize kind"),
            kind
        );
    }

    let custom = SourceKind::custom("dataset").expect("custom kind");
    assert!(custom.is_custom());
    assert_eq!(
        serde_json::to_string(&custom).expect("serialize custom"),
        "\"custom:dataset\""
    );

    let direct_custom = SourceKind::Custom("  dataset  ".to_owned());
    assert_eq!(
        serde_json::to_string(&direct_custom).expect("serialize direct custom"),
        "\"custom:dataset\""
    );
}

#[test]
fn source_kind_rejects_unknown_values_with_core_code() {
    let custom_error = SourceKind::custom("   ").expect_err("empty custom kind");
    assert_eq!(custom_error.code(), "invalid_source_kind");

    let error = serde_json::from_str::<SourceKind>("\"repository\"").expect_err("unknown kind");
    assert!(error.to_string().contains("invalid_source_kind"));

    let invalid_direct_custom = SourceKind::Custom("   ".to_owned());
    let error =
        serde_json::to_string(&invalid_direct_custom).expect_err("invalid custom serialization");
    assert!(error.to_string().contains("invalid_source_kind"));
}

#[test]
fn source_ref_roundtrips_portable_fields() {
    let source = SourceRef::new(SourceKind::Document)
        .with_uri("  urn:higher-graphen:source:1  ")
        .expect("valid uri")
        .with_title("Abstract source")
        .expect("valid title")
        .with_captured_at("2026-04-25T00:00:00Z")
        .expect("valid captured_at")
        .with_source_local_id("section-1")
        .expect("valid source local id");

    let json = serde_json::to_string(&source).expect("serialize source");
    assert_eq!(source.uri.as_deref(), Some("urn:higher-graphen:source:1"));
    let roundtrip: SourceRef = serde_json::from_str(&json).expect("deserialize source");
    assert_eq!(roundtrip, source);
}

#[test]
fn source_ref_rejects_blank_payloads_at_serde_boundaries() {
    assert_eq!(
        SourceRef::new(SourceKind::Document)
            .with_uri("   ")
            .expect_err("empty source uri")
            .code(),
        "malformed_field"
    );

    let direct_invalid = SourceRef {
        kind: SourceKind::Document,
        uri: Some("   ".to_owned()),
        title: None,
        captured_at: None,
        source_local_id: None,
    };
    let error = serde_json::to_string(&direct_invalid).expect_err("invalid source serialization");
    assert!(error.to_string().contains("malformed_field"));

    let malformed = r#"{"kind":"document","title":"   "}"#;
    let error = serde_json::from_str::<SourceRef>(malformed).expect_err("invalid source input");
    assert!(error.to_string().contains("malformed_field"));
}

#[test]
fn severity_and_review_status_have_stable_values_and_order() {
    assert!(Severity::Low < Severity::Medium);
    assert!(Severity::Medium < Severity::High);
    assert!(Severity::High < Severity::Critical);
    assert!(Severity::Critical.is_at_least(Severity::High));
    assert_eq!(Severity::try_from("critical").unwrap(), Severity::Critical);
    assert_eq!(Severity::Critical.as_str(), "critical");
    assert_eq!(Severity::Critical.to_string(), "critical");
    assert_eq!(
        serde_json::to_string(&Severity::Critical).unwrap(),
        "\"critical\""
    );

    assert_eq!(ReviewStatus::default(), ReviewStatus::Unreviewed);
    assert_eq!(
        ReviewStatus::try_from("accepted").unwrap(),
        ReviewStatus::Accepted
    );
    assert_eq!(ReviewStatus::Accepted.as_str(), "accepted");
    assert_eq!(ReviewStatus::Accepted.to_string(), "accepted");
    assert!(ReviewStatus::Accepted.is_accepted());
    assert!(ReviewStatus::Rejected.is_rejected());
    assert!(ReviewStatus::Reviewed.has_review_action());
    assert!(serde_json::from_str::<Severity>("\"urgent\"").is_err());
    assert!(serde_json::from_str::<ReviewStatus>("\"approved\"").is_err());
    assert_eq!(
        Severity::try_from("urgent")
            .expect_err("invalid severity")
            .code(),
        "parse_failure"
    );
}

#[test]
fn provenance_roundtrips_and_requires_review_status_on_input() {
    let source = SourceRef::new(SourceKind::custom("fixture").expect("custom source"));
    let provenance = Provenance::new(source, Confidence::new(0.8).expect("confidence"))
        .with_review_status(ReviewStatus::Unreviewed)
        .with_extraction_method("  manual_fixture  ")
        .expect("valid extraction method")
        .with_extractor_id("extractor-1")
        .expect("valid extractor id")
        .with_notes("keeps review status separate from confidence")
        .expect("valid notes");

    let value = serde_json::to_value(&provenance).expect("serialize provenance");
    assert_eq!(value["review_status"], json!("unreviewed"));
    assert_eq!(value["extraction_method"], json!("manual_fixture"));

    let roundtrip: Provenance = serde_json::from_value(value).expect("deserialize provenance");
    assert_eq!(roundtrip, provenance);

    let malformed = r#"{"source":{"kind":"document"},"confidence":0.8}"#;
    assert!(serde_json::from_str::<Provenance>(malformed).is_err());
}

#[test]
fn provenance_rejects_blank_optional_payloads_at_serde_boundaries() {
    assert_eq!(
        Provenance::new(
            SourceRef::new(SourceKind::Document),
            Confidence::new(0.8).expect("confidence"),
        )
        .with_reviewer_id("   ")
        .expect_err("empty reviewer id")
        .code(),
        "malformed_field"
    );

    let mut direct_invalid = Provenance::new(
        SourceRef::new(SourceKind::Document),
        Confidence::new(0.8).expect("confidence"),
    )
    .with_review_status(ReviewStatus::Reviewed);
    direct_invalid.reviewed_at = Some("   ".to_owned());
    let error =
        serde_json::to_string(&direct_invalid).expect_err("invalid provenance serialization");
    assert!(error.to_string().contains("malformed_field"));

    let malformed = r#"{"source":{"kind":"document"},"confidence":0.8,"review_status":"reviewed","notes":"   "}"#;
    let error =
        serde_json::from_str::<Provenance>(malformed).expect_err("invalid provenance input");
    assert!(error.to_string().contains("malformed_field"));
}

#[test]
fn core_error_exposes_stable_codes_and_roundtrips() {
    let error = Id::new("").expect_err("invalid id");
    assert_eq!(error.code(), "invalid_id");

    let value = serde_json::to_value(&error).expect("serialize error");
    assert_eq!(value["code"], json!("invalid_id"));

    let roundtrip: CoreError = serde_json::from_value(value).expect("deserialize error");
    assert_eq!(roundtrip, error);
}
