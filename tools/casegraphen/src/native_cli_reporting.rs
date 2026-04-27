use serde_json::{json, Value};

const REPORT_SCHEMA: &str = "highergraphen.case.native_cli.report.v1";
const REPORT_TYPE: &str = "native_cli_operation";
const REPORT_VERSION: u32 = 1;

pub(super) fn report(command: &str, result: Value) -> Value {
    json!({
        "schema": REPORT_SCHEMA,
        "report_type": REPORT_TYPE,
        "report_version": REPORT_VERSION,
        "metadata": {
            "command": command,
            "tool_package": "tools/casegraphen",
            "core_packages": [
                "higher-graphen-core"
            ]
        },
        "input": {
            "command": command
        },
        "result": result,
        "projection": {
            "human_review": {
                "summary": "Native CaseGraphen CLI operation completed."
            },
            "ai_view": {
                "operation": command,
                "native_boundary": "CaseSpace plus MorphismLog state is replayed before derived reports are emitted."
            },
            "audit_trace": {
                "source_ids": [],
                "information_loss": [
                    "Native CLI operation reports include the operation result but not a full command-line argv transcript."
                ]
            }
        }
    })
}
