use crate::native_model::{
    CaseSpace, CloseCheck, Projection, NATIVE_CASE_SPACE_SCHEMA, NATIVE_CASE_SPACE_SCHEMA_VERSION,
};
use higher_graphen_core::Id;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const NATIVE_CASE_REPORT_SCHEMA: &str = "highergraphen.case.native.report.v1";
pub const NATIVE_CASE_REPORT_TYPE: &str = "native_case_contract";
pub const NATIVE_CASE_REPORT_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCaseReport {
    pub schema: String,
    pub report_type: String,
    pub report_version: u32,
    pub metadata: NativeReportMetadata,
    pub input: NativeReportInput,
    pub result: NativeReportResult,
    pub projection: NativeReportProjection,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReportMetadata {
    pub command: String,
    pub tool_package: String,
    pub core_packages: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReportInput {
    pub case_space_schema: String,
    pub case_space_schema_version: u32,
    pub case_space_id: Id,
    pub space_id: Id,
    pub revision_id: Id,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReportResult {
    pub case_space: CaseSpace,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_check: Option<CloseCheck>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<NativeOperationEnvelope>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeOperationEnvelope {
    pub operation_id: Id,
    pub operation_type: String,
    pub base_revision_id: Id,
    pub target_revision_id: Id,
    pub accepted: bool,
    pub payload: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReportProjection {
    pub views: Vec<Projection>,
    pub summary: String,
}

pub fn native_case_contract_report(
    command: &str,
    case_space: CaseSpace,
    close_check: Option<CloseCheck>,
    operation: Option<NativeOperationEnvelope>,
    summary: String,
) -> NativeCaseReport {
    let input = NativeReportInput {
        case_space_schema: NATIVE_CASE_SPACE_SCHEMA.to_owned(),
        case_space_schema_version: NATIVE_CASE_SPACE_SCHEMA_VERSION,
        case_space_id: case_space.case_space_id.clone(),
        space_id: case_space.space_id.clone(),
        revision_id: case_space.revision.revision_id.clone(),
    };
    let views = case_space.projections.clone();

    NativeCaseReport {
        schema: NATIVE_CASE_REPORT_SCHEMA.to_owned(),
        report_type: NATIVE_CASE_REPORT_TYPE.to_owned(),
        report_version: NATIVE_CASE_REPORT_VERSION,
        metadata: NativeReportMetadata {
            command: command.to_owned(),
            tool_package: "tools/casegraphen".to_owned(),
            core_packages: vec!["higher-graphen-core".to_owned()],
        },
        input,
        result: NativeReportResult {
            case_space,
            close_check,
            operation,
        },
        projection: NativeReportProjection { views, summary },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NATIVE_EXAMPLE: &str =
        include_str!("../../../schemas/casegraphen/native.case.space.example.json");
    const NATIVE_REPORT_EXAMPLE: &str =
        include_str!("../../../schemas/casegraphen/native.case.report.example.json");

    #[test]
    fn native_report_example_deserializes() {
        let report: NativeCaseReport =
            serde_json::from_str(NATIVE_REPORT_EXAMPLE).expect("native report example");

        assert_eq!(report.schema, NATIVE_CASE_REPORT_SCHEMA);
        assert_eq!(report.report_type, NATIVE_CASE_REPORT_TYPE);
        assert_eq!(
            report.input.case_space_schema,
            "highergraphen.case.space.v1"
        );
    }

    #[test]
    fn native_report_round_trips() {
        let space: CaseSpace =
            serde_json::from_str(NATIVE_EXAMPLE).expect("native case space example");
        let report = native_case_contract_report(
            "casegraphen case inspect",
            space,
            None,
            None,
            "Native case contract fixture.".to_owned(),
        );
        let round_trip: NativeCaseReport =
            serde_json::from_str(&serde_json::to_string(&report).expect("serialize report"))
                .expect("deserialize report");

        assert_eq!(round_trip, report);
    }
}
