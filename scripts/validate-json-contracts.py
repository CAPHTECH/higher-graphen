#!/usr/bin/env python3
"""Validate repository JSON contract coverage and schema conformance.

This check verifies that every schema-bearing fixture resolves to a declared
schema `$id` or alias, validates that fixture against the resolved JSON Schema,
and keeps a lightweight report-envelope check for top-level reports.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

try:
    import jsonschema
except ImportError:  # pragma: no cover - exercised by environments, not tests.
    jsonschema = None  # type: ignore[assignment]

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_ROOT = ROOT / "schemas"
FIXTURE_ROOTS = [ROOT / "schemas", ROOT / "examples"]
ALIAS_PATH = SCHEMA_ROOT / "casegraphen" / "report-schema-aliases.json"

REPORT_REQUIRED_KEYS = {
    "schema",
    "report_type",
    "report_version",
    "metadata",
    "result",
}


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise ContractError(f"{path}: failed to read JSON: {error}") from error
    except json.JSONDecodeError as error:
        raise ContractError(f"{path}: invalid JSON: {error}") from error


def schema_ids() -> dict[str, Path]:
    ids: dict[str, Path] = {}
    for path in sorted(SCHEMA_ROOT.rglob("*.schema.json")):
        value = load_json(path)
        schema_id = value.get("$id") if isinstance(value, dict) else None
        if not isinstance(schema_id, str) or not schema_id:
            raise ContractError(f"{path}: schema file must declare non-empty $id")
        if schema_id in ids:
            raise ContractError(f"{path}: duplicate $id {schema_id!r}; first seen at {ids[schema_id]}")
        ids[schema_id] = path
    return ids


def alias_rules(known_schema_ids: dict[str, Path]) -> list[AliasRule]:
    value = load_json(ALIAS_PATH)
    aliases = value.get("aliases") if isinstance(value, dict) else None
    if not isinstance(aliases, list):
        raise ContractError(f"{ALIAS_PATH}: aliases must be an array")

    rules = []
    for index, alias in enumerate(aliases):
        if not isinstance(alias, dict):
            raise ContractError(f"{ALIAS_PATH}: aliases[{index}] must be an object")
        pattern = alias.get("schema_pattern")
        target = alias.get("target_schema_id")
        if not isinstance(pattern, str) or not isinstance(target, str):
            raise ContractError(
                f"{ALIAS_PATH}: aliases[{index}] needs schema_pattern and target_schema_id"
            )
        if target not in known_schema_ids:
            raise ContractError(
                f"{ALIAS_PATH}: aliases[{index}] targets unknown schema id {target!r}"
            )
        rules.append(AliasRule(re.compile(pattern), target))
    return rules


def json_contract_paths() -> list[Path]:
    paths: list[Path] = []
    for root in FIXTURE_ROOTS:
        paths.extend(
            path
            for path in root.rglob("*.json")
            if path != ALIAS_PATH and not path.name.endswith(".schema.json")
        )
    return sorted(paths)


def top_level_report(path: Path, value: Any) -> tuple[str, dict[str, Any]] | None:
    if not isinstance(value, dict):
        return None
    schema_id = value.get("schema")
    if isinstance(schema_id, str) and schema_id.endswith(".report.v1"):
        return (str(path.relative_to(ROOT)), value)
    return None


def resolve_schema_id(
    report_schema_id: str,
    known_schema_ids: dict[str, Path],
    rules: list[AliasRule],
) -> str | None:
    if report_schema_id in known_schema_ids:
        return report_schema_id
    for rule in rules:
        if rule.pattern.match(report_schema_id):
            return rule.target_schema_id
    return None


def validate_report_shape(location: str, report: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    missing = sorted(REPORT_REQUIRED_KEYS - set(report))
    if missing:
        errors.append(f"{location}: report is missing required keys {missing}")
    if report.get("report_version") != 1:
        errors.append(f"{location}: report_version must be 1")
    metadata = report.get("metadata")
    if not isinstance(metadata, dict):
        errors.append(f"{location}.metadata: expected object")
    else:
        if not metadata.get("command"):
            errors.append(f"{location}.metadata.command: expected non-empty value")
        package_keys = ["tool_package", "runtime_package", "cli_package"]
        if not any(metadata.get(key) for key in package_keys):
            errors.append(
                f"{location}.metadata: expected one of {package_keys} to identify the emitting package"
            )
    return errors


def validate_jsonschema(
    location: str,
    instance: Any,
    schema_id: str,
    known_schema_ids: dict[str, Path],
) -> list[str]:
    if jsonschema is None:
        return [
            "python package 'jsonschema' is required; install it with "
            "`python3 -m pip install jsonschema`"
        ]

    schema_path = known_schema_ids[schema_id]
    schema = load_json(schema_path)
    validator_cls = jsonschema.validators.validator_for(schema)
    validator_cls.check_schema(schema)
    validator = validator_cls(schema)
    validation_errors = sorted(
        validator.iter_errors(instance),
        key=lambda error: [str(part) for part in error.absolute_path],
    )
    return [
        f"{location}: does not validate against {schema_path.relative_to(ROOT)} "
        f"at {json_path(error.absolute_path)}: {error.message}"
        for error in validation_errors
    ]


def json_path(path: Any) -> str:
    parts = list(path)
    if not parts:
        return "$"
    rendered = "$"
    for part in parts:
        if isinstance(part, int):
            rendered += f"[{part}]"
        else:
            rendered += f".{part}"
    return rendered


def main() -> int:
    try:
        known_schema_ids = schema_ids()
        aliases = alias_rules(known_schema_ids)
        errors: list[str] = []
        report_count = 0
        schema_validated_count = 0

        for path in json_contract_paths():
            value = load_json(path)
            location = str(path.relative_to(ROOT))
            schema_value = value.get("schema") if isinstance(value, dict) else None
            if isinstance(schema_value, str):
                target = resolve_schema_id(schema_value, known_schema_ids, aliases)
                if target is None:
                    errors.append(
                        f"{location}: no matching schema $id or explicit alias for {schema_value!r}"
                    )
                else:
                    schema_validated_count += 1
                    errors.extend(
                        validate_jsonschema(location, value, target, known_schema_ids)
                    )

            report_record = top_level_report(path, value)
            if report_record is not None:
                location, report = report_record
                report_count += 1
                report_schema_id = report["schema"]
                target = resolve_schema_id(report_schema_id, known_schema_ids, aliases)
                if target is None:
                    errors.append(
                        f"{location}: no matching schema $id or explicit alias for {report_schema_id!r}"
                    )
                errors.extend(validate_report_shape(location, report))

        if errors:
            print("JSON contract validation failed:", file=sys.stderr)
            for error in errors:
                print(f"- {error}", file=sys.stderr)
            return 1

        print(
            f"validated {len(known_schema_ids)} schema ids, "
            f"{schema_validated_count} schema-bearing fixtures, "
            f"and {report_count} report contracts"
        )
        return 0
    except ContractError as error:
        print(f"JSON contract validation failed: {error}", file=sys.stderr)
        return 1


class ContractError(Exception):
    pass


class AliasRule:
    def __init__(self, pattern: re.Pattern[str], target_schema_id: str) -> None:
        self.pattern = pattern
        self.target_schema_id = target_schema_id


if __name__ == "__main__":
    raise SystemExit(main())
