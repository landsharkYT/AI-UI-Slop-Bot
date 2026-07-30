#!/usr/bin/env bash
set -euo pipefail

binary="${1:-target/release/ai-ui-slop}"
evidence_directory="${2:-target/benchmark-evidence}"
fixture_directory="$(mktemp -d)"
trap 'rm -rf "$fixture_directory"' EXIT
mkdir -p "$evidence_directory"

file_count=500
components_per_file=1000
for file_index in $(seq 1 "$file_count"); do
  fixture="$fixture_directory/Benchmark${file_index}.tsx"
  {
    for component_index in $(seq 1 "$components_per_file"); do
      echo "// deterministic benchmark line ${file_index}:${component_index}"
    done
    echo "export function Benchmark${file_index}(){return <main className=\"p-4\">Benchmark</main>}"
  } > "$fixture"
done

line_count="$(find "$fixture_directory" -type f -name '*.tsx' -print0 | xargs -0 wc -l | tail -1 | awk '{print $1}')"
scripts/measure-command.py \
  "$evidence_directory/metrics.json" \
  "$evidence_directory/report.json" \
  "$evidence_directory/stderr.txt" \
  -- "$binary" scan "$fixture_directory" --format json --progress never
read -r elapsed_ms peak_rss_kib < <(
  python3 -c '
import json
import sys

with open(sys.argv[1], encoding="utf-8") as metrics_file:
    metrics = json.load(metrics_file)
print(metrics["elapsedMs"], metrics.get("peakRssKiB") or 0)
' "$evidence_directory/metrics.json"
)

printf '{"fixtureVersion":"1","fileCount":%s,"lineCount":%s,"elapsedMs":%s,"peakRssKiB":%s,"binary":"%s"}\n' \
  "$file_count" "$line_count" "$elapsed_ms" "${peak_rss_kib:-0}" "$binary" \
  > "$evidence_directory/benchmark.json"
printf 'benchmark evidence: %s\n' "$evidence_directory/benchmark.json"
