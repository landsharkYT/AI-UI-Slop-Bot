#!/usr/bin/env python3
import hashlib
import json
import os
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
binary = Path(sys.argv[1] if len(sys.argv) > 1 else ROOT / "target/release/ai-ui-slop")
fixture = Path(
    sys.argv[2] if len(sys.argv) > 2 else ROOT / "tests/fixtures/recurring-shell"
)
destination = Path(
    sys.argv[3] if len(sys.argv) > 3 else ROOT / "target/progress-evidence.json"
)


def scan(repository: Path, mode: str) -> tuple[int, int, str]:
    started = time.perf_counter_ns()
    completed = subprocess.run(
        [
            str(binary),
            "scan",
            str(repository),
            "--format",
            "json",
            "--progress",
            mode,
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    elapsed = time.perf_counter_ns() - started
    digest = hashlib.sha256(completed.stdout).hexdigest()
    return completed.returncode, elapsed, digest


pairs = []
with tempfile.TemporaryDirectory() as directory:
    repository = Path(directory) / "repository"
    subprocess.run(["cp", "-R", str(fixture), str(repository)], check=True)
    for file_index in range(500):
        lines = [
            f"// deterministic progress workload {file_index}:{line_index}"
            for line_index in range(1000)
        ]
        lines.append(
            f'export function Progress{file_index}(){{return <main className="p-4">Progress</main>}}'
        )
        (repository / f"Progress{file_index}.tsx").write_text(
            "\n".join(lines) + "\n", encoding="utf-8"
        )
    for index in range(20):
        order = ("always", "never") if index % 2 == 0 else ("never", "always")
        results = {mode: scan(repository, mode) for mode in order}
        always = results["always"]
        never = results["never"]
        if always[0] != never[0] or always[2] != never[2]:
            raise SystemExit(f"progress changed behavior in pair {index + 1}")
        delta_percent = (always[1] - never[1]) * 100 / max(never[1], 1)
        pairs.append(
            {
                "pair": index + 1,
                "order": list(order),
                "alwaysNs": always[1],
                "neverNs": never[1],
                "deltaPercent": delta_percent,
                "reportSha256": always[2],
                "exitCode": always[0],
            }
        )

deltas = sorted(pair["deltaPercent"] for pair in pairs)
lower = deltas[max(0, int(len(deltas) * 0.025) - 1)]
upper = deltas[min(len(deltas) - 1, int(len(deltas) * 0.975))]
version = subprocess.run(
    [str(binary), "version"], check=False, capture_output=True, text=True
)
version_lines = version.stdout.splitlines()
version_fields = dict(
    line.split(" ", 1) for line in version_lines[1:] if " " in line
)
revision = subprocess.run(
    ["git", "rev-parse", "HEAD"], check=False, capture_output=True, text=True
).stdout.strip()
resolved_image = os.environ.get("ImageVersion", "local-unqualified")
evidence = {
    "protocolVersion": "1",
    "runnerId": (
        "github-ubuntu-24.04-x64-v1"
        if resolved_image != "local-unqualified"
        else "local-unqualified"
    ),
    "resolvedImageVersion": resolved_image,
    "logicalProcessors": os.cpu_count(),
    "scannerRevision": revision or "unknown",
    "scannerVersion": version_lines[0] if version_lines else "unknown",
    "rulePackVersion": version_fields.get("rule-pack", "unknown"),
    "fixtureVersion": "2",
    "pairs": pairs,
    "medianPairedOverheadPercent": statistics.median(deltas),
    "empirical95PercentInterval": [lower, upper],
    "passesTwoPercentMedianGate": statistics.median(deltas) <= 2,
}
destination.parent.mkdir(parents=True, exist_ok=True)
destination.write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
print(destination)
