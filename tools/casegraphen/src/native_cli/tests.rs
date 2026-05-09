use super::*;
use std::ffi::OsString;

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

#[test]
fn parses_space_commands_as_canonical_native_surface() {
    let command = NativeCliCommand::parse(
        "space",
        args(&[
            "reason",
            "--store",
            "store",
            "--case-space-id",
            "case_space:demo",
            "--format",
            "json",
        ]),
    )
    .expect("space reason command");

    assert!(matches!(
        command,
        NativeCliCommand::CaseReason {
            section: NativeReasonSection::Reason,
            ..
        }
    ));

    let topology = NativeCliCommand::parse(
        "space",
        args(&[
            "topology",
            "diff",
            "--left-store",
            "left",
            "--left-case-space-id",
            "case_space:left",
            "--right-store",
            "right",
            "--right-case-space-id",
            "case_space:right",
            "--format",
            "json",
        ]),
    )
    .expect("space topology diff command");

    assert!(matches!(
        topology,
        NativeCliCommand::CaseTopologyDiff { .. }
    ));
}

#[test]
fn parses_value_namespaces_to_existing_native_operations() {
    assert_value_namespace_reason("obstruction", "list", NativeReasonSection::Obstructions);
    assert_value_namespace_reason("completion", "candidates", NativeReasonSection::Completions);
    assert_projection_namespace();
    assert_invariant_namespace();
    assert_equivalence_namespace();
}

fn assert_value_namespace_reason(namespace: &str, operation: &str, section: NativeReasonSection) {
    let command = NativeCliCommand::parse(
        namespace,
        args(&[
            operation,
            "--store",
            "store",
            "--case-space-id",
            "case_space:demo",
            "--format",
            "json",
        ]),
    )
    .expect("value namespace command");
    assert!(matches!(
        command,
        NativeCliCommand::CaseReason {
            section: parsed,
            ..
        } if parsed == section
    ));
}

fn assert_projection_namespace() {
    let projection = NativeCliCommand::parse(
        "projection",
        args(&[
            "apply",
            "--store",
            "store",
            "--case-space-id",
            "case_space:demo",
            "--projection",
            "projection.json",
            "--format",
            "json",
        ]),
    )
    .expect("projection apply command");
    assert!(matches!(
        projection,
        NativeCliCommand::ProjectionApply { .. }
    ));
}

fn assert_invariant_namespace() {
    let invariant = NativeCliCommand::parse(
        "invariant",
        args(&[
            "check",
            "--store",
            "store",
            "--case-space-id",
            "case_space:demo",
            "--format",
            "json",
        ]),
    )
    .expect("invariant check command");
    assert!(matches!(invariant, NativeCliCommand::InvariantCheck { .. }));
}

fn assert_equivalence_namespace() {
    let equivalence = NativeCliCommand::parse(
        "equivalence",
        args(&[
            "check",
            "--left-store",
            "left",
            "--left-case-space-id",
            "case_space:left",
            "--right-store",
            "right",
            "--right-case-space-id",
            "case_space:right",
            "--format",
            "json",
        ]),
    )
    .expect("equivalence check command");
    assert!(matches!(
        equivalence,
        NativeCliCommand::EquivalenceCheck { .. }
    ));
}

#[test]
fn parses_lift_adapters() {
    let workflow = NativeCliCommand::parse(
        "lift",
        args(&[
            "workflow",
            "--store",
            "store",
            "--input",
            "workflow.graph.json",
            "--revision-id",
            "revision:lifted",
            "--format",
            "json",
        ]),
    )
    .expect("workflow lift command");

    assert!(matches!(
        workflow,
        NativeCliCommand::LiftStructuredSource { adapter, .. } if adapter == "workflow"
    ));

    let native = NativeCliCommand::parse(
        "lift",
        args(&[
            "native",
            "--store",
            "store",
            "--input",
            "native.case.space.json",
            "--revision-id",
            "revision:lifted",
            "--format",
            "json",
        ]),
    )
    .expect("native lift command");
    assert!(matches!(native, NativeCliCommand::CaseImport { .. }));
}
