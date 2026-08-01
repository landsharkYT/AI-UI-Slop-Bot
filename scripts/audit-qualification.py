#!/usr/bin/env python3
import argparse
import json
import re
import sys
from collections import Counter
from pathlib import Path


ALLOWED_STATUSES = {
    "pending",
    "local-pass",
    "hosted-pass",
    "external-pass",
    "full-pass",
    "waived",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Audit qualification evidence recorded in the requirements matrix."
    )
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--format", choices=("text", "json"), default="text")
    parser.add_argument(
        "--require-complete",
        action="store_true",
        help="fail while any requirement remains pending",
    )
    return parser.parse_args()


def matrix_rows(matrix: str) -> list[tuple[str, str, str]]:
    rows = []
    for line in matrix.splitlines():
        match = re.match(
            r"^\| `(?P<requirement>[A-Z][A-Z0-9-]+)` "
            r"\| [^|]* \| [^|]* \| (?P<evidence>.*?) "
            r"\| .* \| (?P<status>[a-z-]+) \|$",
            line,
        )
        if not match:
            continue
        requirement = match.group("requirement")
        evidence_match = re.search(r"`([^`]+)`", match.group("evidence"))
        evidence = evidence_match.group(1) if evidence_match else ""
        rows.append((requirement, evidence, match.group("status")))
    return rows


def main() -> int:
    args = parse_args()
    root = args.root.resolve()
    matrix_path = root / "docs" / "requirements-verification.md"
    try:
        matrix = matrix_path.read_text(encoding="utf-8")
        rows = matrix_rows(matrix)
    except OSError as error:
        print(f"qualification audit: {error}", file=sys.stderr)
        return 1

    failures: list[str] = []
    candidate_ids = re.findall(
        r"^\| `([A-Z][A-Z0-9-]+)` \|", matrix, re.MULTILINE
    )
    parsed_ids = {requirement for requirement, _, _ in rows}
    for requirement in candidate_ids:
        if requirement not in parsed_ids:
            failures.append(f"{requirement}: matrix row could not be parsed")
    if not candidate_ids:
        failures.append("verification matrix contains no requirement rows")
    statuses = Counter(status for _, _, status in rows)
    evidence_files: set[Path] = set()
    for requirement, evidence, status in rows:
        if status not in ALLOWED_STATUSES:
            failures.append(f"{requirement}: unsupported status `{status}`")
            continue
        evidence_path = root / evidence if evidence else None
        if evidence_path is not None and evidence_path.is_file():
            evidence_files.add(evidence_path.resolve())
        if status == "pending":
            continue
        if evidence_path is None or not evidence_path.is_file():
            failures.append(f"{requirement}: evidence file does not exist: {evidence}")
            continue
        try:
            evidence_text = evidence_path.read_text(encoding="utf-8")
        except OSError as error:
            failures.append(f"{requirement}: cannot read evidence: {error}")
            continue
        if requirement not in evidence_text:
            failures.append(f"{requirement}: evidence does not cite requirement ID")

    if args.require_complete and statuses.get("pending", 0):
        failures.append(
            f"{statuses['pending']} requirement(s) remain pending under --require-complete"
        )

    summary = {
        "evidenceFiles": len(evidence_files),
        "invalidRows": len(failures),
        "requirements": len(rows),
        "statuses": dict(sorted(statuses.items())),
        "unverified": statuses.get("pending", 0),
    }
    if args.format == "json":
        print(json.dumps(summary, sort_keys=True, separators=(",", ":")))
    else:
        status_summary = ", ".join(
            f"{status}={count}" for status, count in sorted(statuses.items())
        )
        print(
            f"qualification audit: requirements={len(rows)}, "
            f"evidenceFiles={len(evidence_files)}, {status_summary}"
        )
    for failure in failures:
        print(f"FAIL: {failure}", file=sys.stderr)
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
