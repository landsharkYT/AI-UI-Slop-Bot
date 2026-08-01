#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/release/ai-ui-slop}"
evidence_directory="${2:-target/benchmark-evidence}"
fixture_root="$(mktemp -d)"
trap 'rm -rf "$fixture_root"' EXIT
mkdir -p "$evidence_directory"

create_representative_files() {
  local directory="$1"
  mkdir -p "$directory"
  for file_index in $(seq 1 2000); do
    local extension="tsx"
    if (( file_index % 2 == 0 )); then extension="jsx"; fi
    local padding="p-$((file_index % 8 + 1))"
    printf 'export function Component%s(){return <section className="%s border shadow-sm">Component %s</section>}\n' \
      "$file_index" "$padding" "$file_index" > "$directory/Component${file_index}.${extension}"
  done
}

create_representative_lines() {
  local directory="$1"
  mkdir -p "$directory"
  for file_index in $(seq 1 500); do
    local fixture="$directory/Benchmark${file_index}.tsx"
    for line_index in $(seq 1 999); do
      printf '// deterministic benchmark line %s:%s\n' "$file_index" "$line_index"
    done > "$fixture"
    printf 'export function Benchmark%s(){return <main className="p-4">Benchmark</main>}\n' \
      "$file_index" >> "$fixture"
  done
}

run_workload() {
  local workload_id="$1"
  local directory="$2"
  local result_directory="$evidence_directory/$workload_id"
  mkdir -p "$result_directory"
  scripts/measure-command.py \
    "$result_directory/metrics.json" \
    "$result_directory/report.json" \
    "$result_directory/stderr.txt" \
    -- "$binary" scan "$directory" --format json --progress never
  local file_count
  local line_count
  file_count="$(find "$directory" -type f \( -name '*.jsx' -o -name '*.tsx' \) | wc -l)"
  line_count="$(find "$directory" -type f \( -name '*.jsx' -o -name '*.tsx' \) -print0 | xargs -0 wc -l | tail -1 | awk '{print $1}')"
  python3 - "$workload_id" "$file_count" "$line_count" \
    "$result_directory/metrics.json" "$result_directory/workload.json" <<'PY'
import json
import sys
from pathlib import Path

workload_id, file_count, line_count, metrics_path, output_path = sys.argv[1:]
metrics = json.loads(Path(metrics_path).read_text(encoding="utf-8"))
evidence = {
    "id": workload_id,
    "fileCount": int(file_count),
    "lineCount": int(line_count),
    "elapsedMilliseconds": metrics["elapsedMs"],
    "peakRssKiB": metrics["peakRssKiB"],
    "exitCode": metrics["exitCode"],
    "passesElapsedGate": metrics["elapsedMs"] <= 60_000,
    "passesMemoryGate": metrics["peakRssKiB"] <= 1_572_864,
}
Path(output_path).write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
if metrics["elapsedMs"] > 60_000 or metrics["peakRssKiB"] > 1_572_864:
    raise SystemExit("reference benchmark exceeded a Full V1 performance gate")
PY
}

files_fixture="$fixture_root/representative-files"
lines_fixture="$fixture_root/representative-lines"
create_representative_files "$files_fixture"
create_representative_lines "$lines_fixture"
run_workload "representative-files" "$files_fixture"
run_workload "representative-lines" "$lines_fixture"

python3 - "$evidence_directory" "$binary" <<'PY'
import json
import os
import platform
import subprocess
import sys
from pathlib import Path

destination = Path(sys.argv[1])
binary = sys.argv[2]

def output(command):
    return subprocess.run(command, check=False, capture_output=True, text=True).stdout.strip()

cpu_model = "unknown"
for line in Path("/proc/cpuinfo").read_text(encoding="utf-8").splitlines():
    if line.startswith("model name"):
        cpu_model = line.split(":", 1)[1].strip()
        break

version_lines = output([binary, "version"]).splitlines()
version_fields = dict(
    line.split(" ", 1) for line in version_lines[1:] if " " in line
)
resolved_image = os.environ.get("ImageVersion", "local-unqualified")

evidence = {
    "protocolVersion": "1",
    "runnerId": "github-ubuntu-24.04-x64-v1" if resolved_image != "local-unqualified" else "local-unqualified",
    "resolvedImageVersion": resolved_image,
    "osRelease": output(["sh", "-c", ". /etc/os-release && printf %s \"$PRETTY_NAME\""]),
    "kernel": platform.release(),
    "cpuModel": cpu_model,
    "logicalProcessors": os.cpu_count(),
    "memoryBytes": int(output(["awk", "/MemTotal/ {print $2 * 1024}", "/proc/meminfo"])),
    "rustcVersion": output(["rustc", "--version"]),
    "cargoVersion": output(["cargo", "--version"]),
    "scannerRevision": output(["git", "rev-parse", "HEAD"]),
    "scannerVersion": version_lines[0] if version_lines else "unknown",
    "rulePackVersion": version_fields.get("rule-pack", "unknown"),
    "scannerOptions": ["scan", "<fixture>", "--format", "json", "--progress", "never"],
    "fixtureVersion": "2",
    "workloads": [
        json.loads((destination / "representative-files/workload.json").read_text(encoding="utf-8")),
        json.loads((destination / "representative-lines/workload.json").read_text(encoding="utf-8")),
    ],
}
(destination / "benchmark.json").write_text(
    json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)
PY
printf 'benchmark evidence: %s\n' "$evidence_directory/benchmark.json"
