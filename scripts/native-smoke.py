#!/usr/bin/env python3
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def command_output(command: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run(command, check=False, capture_output=True)


def main() -> int:
    if len(sys.argv) != 4:
        print("usage: native-smoke.py BINARY TARGET OUTPUT", file=sys.stderr)
        return 2
    binary = Path(sys.argv[1]).resolve()
    target = sys.argv[2]
    output = Path(sys.argv[3])
    if not binary.is_file():
        print(f"native smoke: binary does not exist: {binary}", file=sys.stderr)
        return 1

    version = command_output([str(binary), "version"])
    version_lines = version.stdout.decode("utf-8", errors="replace").splitlines()
    version_fields = dict(
        line.split(" ", 1) for line in version_lines[1:] if " " in line
    )
    report_hashes: list[str] = []
    scan_codes: list[int] = []
    for _ in range(2):
        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory) / "repository"
            shutil.copytree(ROOT / "tests" / "fixtures" / "recurring-shell", repository)
            scan = command_output(
                [
                    str(binary),
                    "scan",
                    str(repository),
                    "--format",
                    "json",
                    "--progress",
                    "never",
                ]
            )
            scan_codes.append(scan.returncode)
            report_hashes.append(hashlib.sha256(scan.stdout).hexdigest())

    revision = command_output(["git", "rev-parse", "HEAD"])
    evidence = {
        "schemaVersion": "1",
        "target": target,
        "runnerOs": platform.system(),
        "runnerArch": platform.machine(),
        "resolvedImageVersion": os.environ.get("ImageVersion", "local-unqualified"),
        "scannerRevision": revision.stdout.decode("utf-8", errors="replace").strip()
        or "unknown",
        "scannerVersion": version_lines[0] if version_lines else "unknown",
        "rulePackVersion": version_fields.get("rule-pack", "unknown"),
        "binarySha256": digest(binary),
        "binaryBytes": binary.stat().st_size,
        "versionExitCode": version.returncode,
        "scanExitCode": scan_codes[0] if len(set(scan_codes)) == 1 else None,
        "reportSha256": report_hashes,
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    failures = []
    if version.returncode != 0:
        failures.append(f"version smoke failed with exit {version.returncode}")
    if scan_codes != [0, 0]:
        failures.append(f"scan smoke failed with exits {scan_codes}")
    if len(set(report_hashes)) != 1:
        failures.append("repeated native scan output was not deterministic")
    for failure in failures:
        print(f"native smoke: {failure}", file=sys.stderr)
    if not failures:
        print(f"native smoke evidence: {output}")
    return int(bool(failures))


if __name__ == "__main__":
    raise SystemExit(main())
