#!/usr/bin/env python3
"""Smoke check the provider-neutral HigherGraphen CLI skill bundle."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
BUNDLE_DIR = Path(__file__).resolve().parent
METADATA = BUNDLE_DIR / "bundle.json"

EXPECTED_SCHEMA = "highergraphen.cli_skill_bundle.v1"
EXPECTED_OUT_OF_SCOPE = {
    "mcp_server",
    "provider_marketplace_publication",
    "provider_sdk_integration",
    "provider_specific_manifest",
}

REQUIRED_METADATA_PATHS = [
    ("contract_references", "cli_reference"),
    ("contract_references", "agent_handoff"),
    ("contract_references", "agent_integration_spec"),
    ("contract_references", "report_schema"),
    ("contract_references", "report_fixture"),
    ("contract_references", "ddd_review_contract"),
    ("contract_references", "ddd_review_input_schema"),
    ("contract_references", "ddd_review_input_fixture"),
    ("contract_references", "ddd_review_report_schema"),
    ("contract_references", "ddd_review_report_fixture"),
    ("contract_references", "ddd_review_legacy_example"),
    ("contract_references", "ddd_review_legacy_case_space_fixture"),
    ("contract_references", "ddd_review_source_skill"),
    ("contract_references", "contract_validator"),
    ("contract_references", "bundle_contract_reference"),
]

ARCHITECTURE_REVIEW_TERMS = [
    "highergraphen architecture smoke direct-db-access",
    "scripts/validate-cli-report-contract.py",
    "violation_detected",
    "review_status: \"unreviewed\"",
    "deterministic smoke coverage",
    "projection.information_loss",
]

HIGHERGRAPHEN_DDD_SKILL_TERMS = [
    "highergraphen-ddd",
    "highergraphen ddd review",
    "ddd input from-case-space",
    "bounded context",
    "domain model",
    "boundary",
    "evidence boundaries",
    "projection loss",
    "closeability",
    "sales-billing-customer.case.space.json",
    "AI-inferred",
]

HIGHERGRAPHEN_DDD_CONTRACT_TERMS = [
    "highergraphen ddd review",
    "ddd input from-case-space",
    "bounded context",
    "domain model",
    "boundary",
    "evidence boundaries",
    "projection loss",
    "closeability",
    "sales-billing-customer.case.space.json",
    "AI-inferred",
]

FORBIDDEN_MANIFEST_NAMES = {
    "claude.json",
    "codex.json",
    "manifest.json",
    "marketplace.json",
    "mcp.json",
    "openai.json",
    "plugin.json",
}


class BundleError(Exception):
    """Raised when the bundle fails its smoke check."""


def load_metadata() -> dict[str, Any]:
    try:
        data = json.loads(METADATA.read_text(encoding="utf-8"))
    except OSError as error:
        raise BundleError(f"{METADATA}: failed to read metadata: {error}") from error
    except json.JSONDecodeError as error:
        raise BundleError(f"{METADATA}: invalid JSON: {error}") from error

    if not isinstance(data, dict):
        raise BundleError(f"{METADATA}: top-level metadata must be an object")
    return data


def require_equal(errors: list[str], label: str, actual: Any, expected: Any) -> None:
    if actual != expected:
        errors.append(f"{label}: expected {expected!r}, got {actual!r}")


def require_path(errors: list[str], relative: str) -> Path:
    path = ROOT / relative
    if not path.exists():
        errors.append(f"{relative}: referenced file does not exist")
    return path


def check_metadata(metadata: dict[str, Any]) -> list[str]:
    errors: list[str] = []

    require_equal(errors, "schema", metadata.get("schema"), EXPECTED_SCHEMA)
    require_equal(errors, "provider_neutral", metadata.get("provider_neutral"), True)
    require_equal(
        errors,
        "distribution_mode",
        metadata.get("distribution_mode"),
        "cli_skill_bundle",
    )

    out_of_scope = metadata.get("out_of_scope")
    if set(out_of_scope or []) != EXPECTED_OUT_OF_SCOPE:
        errors.append("out_of_scope: missing required boundary entries")

    for section, key in REQUIRED_METADATA_PATHS:
        relative = metadata.get(section, {}).get(key)
        if not isinstance(relative, str):
            errors.append(f"{section}.{key}: expected repository-relative path")
            continue
        require_path(errors, relative)

    skills = metadata.get("skills")
    if not isinstance(skills, list) or len(skills) != 3:
        errors.append(
            "skills: expected highergraphen, highergraphen-ddd, and architecture-review"
        )
        return errors

    skill_names = {skill.get("name") for skill in skills if isinstance(skill, dict)}
    if skill_names != {
        "highergraphen",
        "highergraphen-ddd",
        "architecture-review",
    }:
        errors.append(f"skills: unexpected names {sorted(skill_names)!r}")

    for skill in skills:
        if not isinstance(skill, dict):
            errors.append("skills: entries must be objects")
            continue
        source = skill.get("source")
        if not isinstance(source, str):
            errors.append(f"{skill.get('name')}: missing source")
            continue
        require_path(errors, source)

    return errors


def check_highergraphen_skill_sync(metadata: dict[str, Any]) -> list[str]:
    return check_source_skill(metadata, "highergraphen")


def check_highergraphen_ddd_skill_sync(metadata: dict[str, Any]) -> list[str]:
    return check_source_skill(metadata, "highergraphen-ddd")


def check_source_skill(metadata: dict[str, Any], name: str) -> list[str]:
    skill = find_skill(metadata, name)
    if skill is None:
        return [f"{name} skill metadata is missing"]

    source = require_existing_path(skill, "source")
    if not str(source.relative_to(ROOT)).startswith("skills/"):
        return [f"{name} skill source must live under skills/"]
    return []


def check_architecture_review_skill(metadata: dict[str, Any]) -> list[str]:
    skill = find_skill(metadata, "architecture-review")
    if skill is None:
        return ["architecture-review skill metadata is missing"]

    path = require_existing_path(skill, "source")
    text = path.read_text(encoding="utf-8")
    return [
        f"{path.relative_to(ROOT)}: missing {term!r}"
        for term in ARCHITECTURE_REVIEW_TERMS
        if term not in text
    ]


def check_highergraphen_ddd_surface(metadata: dict[str, Any]) -> list[str]:
    errors: list[str] = []

    skill = find_skill(metadata, "highergraphen-ddd")
    if skill is None:
        return ["highergraphen-ddd skill metadata is missing"]

    skill_path = require_existing_path(skill, "source")
    errors.extend(missing_terms(skill_path, HIGHERGRAPHEN_DDD_SKILL_TERMS))

    references = metadata.get("contract_references", {})
    if not isinstance(references, dict):
        return errors + ["contract_references: expected object"]

    example_path = require_existing_path(
        {
            "name": "ddd_review_legacy_example",
            "source": references.get("ddd_review_legacy_example"),
        },
        "source",
    )
    errors.extend(missing_terms(example_path, ["Sales/Billing", "Customer"]))

    fixture_path = require_existing_path(
        {
            "name": "ddd_review_legacy_case_space_fixture",
            "source": references.get("ddd_review_legacy_case_space_fixture"),
        },
        "source",
    )
    errors.extend(missing_terms(fixture_path, ["case_space:ddd-sales-billing-demo"]))

    contract_path = require_existing_path(
        {
            "name": "ddd_review_contract",
            "source": references.get("ddd_review_contract"),
        },
        "source",
    )
    errors.extend(missing_terms(contract_path, HIGHERGRAPHEN_DDD_CONTRACT_TERMS))

    return errors


def find_skill(metadata: dict[str, Any], name: str) -> dict[str, Any] | None:
    for item in metadata.get("skills", []):
        if isinstance(item, dict) and item.get("name") == name:
            return item
    return None


def require_existing_path(metadata: dict[str, Any], key: str) -> Path:
    relative = metadata.get(key)
    if not isinstance(relative, str):
        raise BundleError(f"{metadata.get('name')}.{key}: expected path")
    path = ROOT / relative
    if not path.exists():
        raise BundleError(f"{relative}: referenced file does not exist")
    return path


def missing_terms(path: Path, terms: list[str]) -> list[str]:
    text = path.read_text(encoding="utf-8")
    return [
        f"{path.relative_to(ROOT)}: missing {term!r}"
        for term in terms
        if term not in text
    ]


def check_provider_specific_files() -> list[str]:
    errors: list[str] = []
    for path in BUNDLE_DIR.rglob("*"):
        if path.is_file() and path.name in FORBIDDEN_MANIFEST_NAMES:
            errors.append(f"{path.relative_to(ROOT)}: provider-specific manifest found")
    return errors


def main() -> int:
    try:
        metadata = load_metadata()
        errors = []
        errors.extend(check_metadata(metadata))
        errors.extend(check_highergraphen_skill_sync(metadata))
        errors.extend(check_highergraphen_ddd_skill_sync(metadata))
        errors.extend(check_architecture_review_skill(metadata))
        errors.extend(check_highergraphen_ddd_surface(metadata))
        errors.extend(check_provider_specific_files())
    except BundleError as error:
        errors = [str(error)]

    if errors:
        print("cli skill bundle smoke check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("cli skill bundle smoke check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
