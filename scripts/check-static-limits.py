#!/usr/bin/env python3
"""Local static-analysis checks for HigherGraphen.

The checker intentionally avoids third-party dependencies so it can run before
the workspace has a larger toolchain contract. It enforces hard limits from the
static analysis policy and catches obvious dependency-direction violations.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CRATES = ROOT / "crates"
TOOLS = ROOT / "tools"

MAX_RUST_FILE_LINES = 700
MAX_RUST_FUNCTION_LINES = 80
MAX_DECISION_POINTS = 12

PACKAGE_ORDER = [
    "higher-graphen-core",
    "higher-graphen-structure",
    "higher-graphen-projection",
    "higher-graphen-evidence",
    "higher-graphen-reasoning",
    "higher-graphen-interpretation",
    "higher-graphen-runtime",
]

TOOL_PACKAGES = {
    "casegraphen",
    "highergraphen-cli",
}

ORDER_INDEX = {name: index for index, name in enumerate(PACKAGE_ORDER)}
DEPENDENCY_SECTIONS = {
    "dependencies",
    "dev-dependencies",
    "build-dependencies",
}

FUNCTION_START = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+|async\s+|unsafe\s+|extern\s+)*fn\s+\w+"
)
DECISION_TOKENS = re.compile(r"\b(if|else\s+if|match|for|while|loop)\b|&&|\|\|")


def logical_lines(lines: list[str]) -> list[str]:
    return [
        line
        for line in lines
        if line.strip() and not line.lstrip().startswith("//")
    ]


def brace_delta(line: str) -> int:
    return line.count("{") - line.count("}")


def check_rust_file(path: Path) -> list[str]:
    errors: list[str] = []
    lines = path.read_text(encoding="utf-8").splitlines()
    logical = logical_lines(lines)

    if len(logical) > MAX_RUST_FILE_LINES:
        errors.append(
            f"{path}: {len(logical)} logical lines exceeds hard limit "
            f"{MAX_RUST_FILE_LINES}"
        )

    index = 0
    while index < len(lines):
        line = lines[index]
        if not FUNCTION_START.search(line):
            index += 1
            continue

        start_line = index + 1
        body_lines = [line]
        depth = brace_delta(line)
        index += 1

        while index < len(lines):
            current = lines[index]
            body_lines.append(current)
            depth += brace_delta(current)
            index += 1
            if depth <= 0 and "{" in "".join(body_lines):
                break

        body_logical = logical_lines(body_lines)
        decisions = sum(len(DECISION_TOKENS.findall(item)) for item in body_lines)

        if len(body_logical) > MAX_RUST_FUNCTION_LINES:
            errors.append(
                f"{path}:{start_line}: function has {len(body_logical)} "
                f"logical lines, hard limit is {MAX_RUST_FUNCTION_LINES}"
            )
        if decisions > MAX_DECISION_POINTS:
            errors.append(
                f"{path}:{start_line}: function has {decisions} decision "
                f"points, hard limit is {MAX_DECISION_POINTS}"
            )

    return errors


def dependency_section(header: str) -> str | None:
    normalized = header.strip().strip("[]")
    if normalized in DEPENDENCY_SECTIONS:
        return normalized
    if normalized.startswith("target.") and normalized.endswith(".dependencies"):
        return "dependencies"
    return None


def check_manifest(path: Path) -> list[str]:
    package = path.parent.name

    if path.parent.parent == TOOLS:
        if package not in TOOL_PACKAGES:
            return [f"{path}: tool package {package} is missing from TOOL_PACKAGES"]
        return check_tool_manifest(path)

    if path.parent.parent == CRATES and package not in ORDER_INDEX:
        return [f"{path}: reusable crate {package} is missing from PACKAGE_ORDER"]

    if package not in ORDER_INDEX:
        return []

    errors: list[str] = []
    current_section: str | None = None
    current_index = ORDER_INDEX[package]

    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped.startswith("[") and stripped.endswith("]"):
            current_section = dependency_section(stripped)
            continue
        if current_section is None or "=" not in stripped:
            continue

        dependency_name = stripped.split("=", 1)[0].strip().strip('"')
        if dependency_name not in ORDER_INDEX:
            continue

        dependency_index = ORDER_INDEX[dependency_name]
        if dependency_index >= current_index:
            errors.append(
                f"{path}: {package} must not depend on downstream or lateral "
                f"crate {dependency_name}"
            )

    return errors


def check_tool_manifest(path: Path) -> list[str]:
    errors: list[str] = []
    current_section: str | None = None

    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped.startswith("[") and stripped.endswith("]"):
            current_section = dependency_section(stripped)
            continue
        if current_section is None or "=" not in stripped:
            continue

        dependency_name = stripped.split("=", 1)[0].strip().strip('"')
        if dependency_name in TOOL_PACKAGES:
            errors.append(f"{path}: tools must not depend on other tool packages")

    return errors


def check_workspace_dependency_boundaries() -> list[str]:
    completed = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if completed.returncode != 0:
        return [f"cargo metadata failed: {completed.stderr.strip()}"]

    metadata = json.loads(completed.stdout)
    package_paths = {
        package["name"]: Path(package["manifest_path"]).parent.relative_to(ROOT)
        for package in metadata["packages"]
    }

    errors: list[str] = []
    for package in metadata["packages"]:
        package_name = package["name"]
        package_path = package_paths[package_name]
        package_root = package_path.parts[0] if package_path.parts else ""

        for dependency in package["dependencies"]:
            dependency_name = dependency["name"]
            dependency_path = package_paths.get(dependency_name)
            if dependency_path is None:
                continue

            dependency_root = dependency_path.parts[0] if dependency_path.parts else ""
            if package_root == "crates" and dependency_root in {"tools", "examples"}:
                errors.append(
                    f"{package_path / 'Cargo.toml'}: reusable crate {package_name} "
                    f"must not depend on workspace {dependency_root} package {dependency_name}"
                )
            if package_root == "tools" and dependency_root == "tools":
                errors.append(
                    f"{package_path / 'Cargo.toml'}: tools must not depend on "
                    f"other tool packages ({dependency_name})"
                )

    return errors


def main() -> int:
    errors: list[str] = []

    for rust_file in sorted(CRATES.glob("higher-graphen-*/src/**/*.rs")):
        errors.extend(check_rust_file(rust_file))
    for rust_file in sorted(TOOLS.glob("*/src/**/*.rs")):
        errors.extend(check_rust_file(rust_file))

    for manifest in sorted(CRATES.glob("higher-graphen-*/Cargo.toml")):
        errors.extend(check_manifest(manifest))
    for manifest in sorted(TOOLS.glob("*/Cargo.toml")):
        errors.extend(check_manifest(manifest))
    errors.extend(check_workspace_dependency_boundaries())

    if errors:
        print("static analysis policy violations:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("static analysis policy checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
