#!/usr/bin/env python3
import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
requirements = (ROOT / "requirements.md").read_text(encoding="utf-8")
matrix = (ROOT / "docs/requirements-verification.md").read_text(encoding="utf-8")

failures: list[str] = []
requirement_ids = re.findall(r"\*\*\[([A-Z][A-Z0-9-]+)\]\*\*", requirements)
matrix_ids = re.findall(r"^\| `([A-Z][A-Z0-9-]+)` \|", matrix, re.MULTILINE)

for label, values in [("requirements", requirement_ids), ("matrix", matrix_ids)]:
    duplicates = sorted({value for value in values if values.count(value) > 1})
    if duplicates:
        failures.append(f"duplicate {label} IDs: {', '.join(duplicates)}")

missing = sorted(set(requirement_ids) - set(matrix_ids))
extra = sorted(set(matrix_ids) - set(requirement_ids))
if missing:
    failures.append(f"IDs missing from matrix: {', '.join(missing)}")
if extra:
    failures.append(f"matrix IDs absent from requirements: {', '.join(extra)}")

source_digest = hashlib.sha256(requirements.encode()).hexdigest()
declared_digest = re.search(r"Source: `requirements\.md` SHA-256 `([0-9a-f]{64})`", matrix)
if not declared_digest or declared_digest.group(1) != source_digest:
    failures.append(
        f"verification matrix digest is stale: expected {source_digest}"
    )

for schema in sorted((ROOT / "schemas").glob("*.json")):
    try:
        json.loads(schema.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        failures.append(f"invalid JSON schema {schema.relative_to(ROOT)}: {error}")

for adr in sorted((ROOT / "docs/adr").glob("*.md")):
    text = adr.read_text(encoding="utf-8")
    if not re.search(r"^Status: (Proposed|Accepted|Superseded|Rejected)$", text, re.MULTILINE):
        failures.append(f"missing or invalid ADR status: {adr.relative_to(ROOT)}")

if failures:
    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)
    raise SystemExit(1)

print(
    f"requirements audit: {len(requirement_ids)} IDs, exact matrix coverage, schemas and ADRs valid"
)
