#!/usr/bin/env python3
"""Validate the first HigherGraphen CLI report contract.

The checker intentionally uses only the Python standard library. It supports
the JSON Schema keywords used by the repository-owned v1 report schema, then
adds semantic checks that matter to agent skills.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SCHEMA = (
    ROOT
    / "schemas"
    / "reports"
    / "architecture-direct-db-access-smoke.report.schema.json"
)

CLI_COMMAND = [
    "cargo",
    "run",
    "-q",
    "-p",
    "highergraphen-cli",
    "--",
    "architecture",
    "smoke",
    "direct-db-access",
    "--format",
    "json",
]


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise ContractError(f"{path}: failed to read JSON: {error}") from error
    except json.JSONDecodeError as error:
        raise ContractError(f"{path}: invalid JSON: {error}") from error


def run_cli_report() -> Any:
    completed = subprocess.run(
        CLI_COMMAND,
        cwd=ROOT,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if completed.returncode != 0:
        raise ContractError(
            "CLI report command failed with exit code "
            f"{completed.returncode}\n{completed.stderr.strip()}"
        )

    lines = completed.stdout.splitlines()
    if len(lines) != 1:
        raise ContractError(
            "CLI stdout must contain exactly one JSON report line; "
            f"found {len(lines)} lines"
        )

    try:
        return json.loads(lines[0])
    except json.JSONDecodeError as error:
        raise ContractError(f"CLI stdout is not valid JSON: {error}") from error


def validate_json_schema(instance: Any, schema: Any, root: Any) -> list[str]:
    return validate_value(instance, schema, root, "$")


def validate_value(instance: Any, schema: Any, root: Any, path: str) -> list[str]:
    if schema is True:
        return []
    if schema is False:
        return [f"{path}: schema disallows this value"]
    if not isinstance(schema, dict):
        return [f"{path}: invalid schema node {schema!r}"]

    errors: list[str] = []
    ref = schema.get("$ref")
    if ref is not None:
        errors.extend(validate_value(instance, resolve_ref(ref, root), root, path))

    if "type" in schema:
        expected_type = schema["type"]
        if not type_matches(instance, expected_type):
            errors.append(f"{path}: expected type {expected_type!r}")

    if "const" in schema and instance != schema["const"]:
        errors.append(f"{path}: expected constant {schema['const']!r}, got {instance!r}")

    if "enum" in schema and instance not in schema["enum"]:
        errors.append(f"{path}: expected one of {schema['enum']!r}, got {instance!r}")

    if isinstance(instance, str) and len(instance) < schema.get("minLength", 0):
        errors.append(f"{path}: string is shorter than minLength {schema['minLength']}")

    if isinstance(instance, list):
        errors.extend(validate_array(instance, schema, root, path))

    if isinstance(instance, dict):
        errors.extend(validate_object(instance, schema, root, path))

    return errors


def validate_array(instance: list[Any], schema: dict[str, Any], root: Any, path: str) -> list[str]:
    errors: list[str] = []

    if "minItems" in schema and len(instance) < schema["minItems"]:
        errors.append(f"{path}: array has fewer than {schema['minItems']} items")
    if "maxItems" in schema and len(instance) > schema["maxItems"]:
        errors.append(f"{path}: array has more than {schema['maxItems']} items")

    prefix_items = schema.get("prefixItems", [])
    for index, item_schema in enumerate(prefix_items):
        if index < len(instance):
            errors.extend(validate_value(instance[index], item_schema, root, f"{path}[{index}]"))

    items_schema = schema.get("items")
    if items_schema is False and len(instance) > len(prefix_items):
        errors.append(f"{path}: array has items beyond fixed prefix")
    elif items_schema is not None and items_schema is not False:
        start = len(prefix_items) if prefix_items else 0
        for index in range(start, len(instance)):
            errors.extend(validate_value(instance[index], items_schema, root, f"{path}[{index}]"))

    if "contains" in schema:
        contains_schema = schema["contains"]
        if not any(
            not validate_value(item, contains_schema, root, f"{path}[{index}]")
            for index, item in enumerate(instance)
        ):
            errors.append(f"{path}: no item matches contains schema")

    return errors


def validate_object(instance: dict[str, Any], schema: dict[str, Any], root: Any, path: str) -> list[str]:
    errors: list[str] = []
    properties = schema.get("properties", {})

    for key in schema.get("required", []):
        if key not in instance:
            errors.append(f"{path}: missing required property {key!r}")

    if schema.get("additionalProperties") is False:
        extra_keys = sorted(set(instance) - set(properties))
        for key in extra_keys:
            errors.append(f"{path}: unexpected property {key!r}")

    for key, property_schema in properties.items():
        if key in instance:
            errors.extend(validate_value(instance[key], property_schema, root, f"{path}.{key}"))

    return errors


def resolve_ref(ref: str, root: Any) -> Any:
    if not ref.startswith("#/"):
        raise ContractError(f"unsupported schema reference {ref!r}")

    current = root
    for part in ref[2:].split("/"):
        key = part.replace("~1", "/").replace("~0", "~")
        try:
            current = current[key]
        except (KeyError, TypeError) as error:
            raise ContractError(f"unresolvable schema reference {ref!r}") from error
    return current


def type_matches(instance: Any, expected_type: Any) -> bool:
    if isinstance(expected_type, list):
        return any(type_matches(instance, item) for item in expected_type)
    if expected_type == "object":
        return isinstance(instance, dict)
    if expected_type == "array":
        return isinstance(instance, list)
    if expected_type == "string":
        return isinstance(instance, str)
    if expected_type == "integer":
        return isinstance(instance, int) and not isinstance(instance, bool)
    if expected_type == "number":
        return isinstance(instance, (int, float)) and not isinstance(instance, bool)
    if expected_type == "boolean":
        return isinstance(instance, bool)
    if expected_type == "null":
        return instance is None
    return False


def validate_semantics(report: dict[str, Any], schema: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    expected_schema_id = schema.get("$id")

    require_equal(errors, "$.schema", report.get("schema"), expected_schema_id)
    require_equal(errors, "$.report_type", report.get("report_type"), "architecture_direct_db_access_smoke")
    require_equal(errors, "$.report_version", report.get("report_version"), 1)
    require_equal(errors, "$.metadata.command", dig(report, "metadata", "command"), "highergraphen architecture smoke direct-db-access")
    require_equal(errors, "$.metadata.cli_package", dig(report, "metadata", "cli_package"), "highergraphen-cli")
    require_equal(errors, "$.result.status", dig(report, "result", "status"), "violation_detected")

    obstructions = dig(report, "result", "obstructions")
    if not isinstance(obstructions, list) or len(obstructions) != 1:
        errors.append("$.result.obstructions: expected exactly one direct database access obstruction")
    else:
        require_equal(
            errors,
            "$.result.obstructions[0].provenance.review_status",
            dig(obstructions[0], "provenance", "review_status"),
            "unreviewed",
        )

    candidates = dig(report, "result", "completion_candidates")
    if not isinstance(candidates, list) or len(candidates) != 1:
        errors.append("$.result.completion_candidates: expected exactly one completion candidate")
    else:
        candidate = candidates[0]
        require_equal(errors, "$.result.completion_candidates[0].id", candidate.get("id"), "candidate:billing-status-api")
        require_equal(
            errors,
            "$.result.completion_candidates[0].review_status",
            candidate.get("review_status"),
            "unreviewed",
        )
        suggested_id = dig(candidate, "suggested_structure", "structure_id")
        require_equal(
            errors,
            "$.result.completion_candidates[0].suggested_structure.structure_id",
            suggested_id,
            "cell:billing-status-api",
        )
        cell_ids = {
            cell.get("id")
            for cell in dig(report, "scenario", "cells", default=[])
            if isinstance(cell, dict)
        }
        if suggested_id in cell_ids:
            errors.append("completion candidate cell was promoted into accepted scenario cells")

    return errors


def require_equal(errors: list[str], path: str, actual: Any, expected: Any) -> None:
    if actual != expected:
        errors.append(f"{path}: expected {expected!r}, got {actual!r}")


def dig(value: Any, *keys: str, default: Any = None) -> Any:
    current = value
    for key in keys:
        if not isinstance(current, dict) or key not in current:
            return default
        current = current[key]
    return current


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--report",
        type=Path,
        help="Validate an existing JSON report instead of running the CLI.",
    )
    parser.add_argument(
        "--schema",
        type=Path,
        default=DEFAULT_SCHEMA,
        help="Report schema path.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    schema = load_json(args.schema)
    report = load_json(args.report) if args.report else run_cli_report()

    errors = validate_json_schema(report, schema, schema)
    if isinstance(report, dict) and isinstance(schema, dict):
        errors.extend(validate_semantics(report, schema))

    if errors:
        print("CLI report contract validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    source = str(args.report) if args.report else "generated CLI stdout"
    print(f"validated {source} against {args.schema}")
    print(
        "stable semantics ok: violation_detected, one obstruction, "
        "one unreviewed completion candidate, candidate not accepted structure"
    )
    return 0


class ContractError(Exception):
    """Raised when validation cannot run or the CLI report is malformed."""


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ContractError as error:
        print(f"CLI report contract validation failed: {error}", file=sys.stderr)
        raise SystemExit(1)
