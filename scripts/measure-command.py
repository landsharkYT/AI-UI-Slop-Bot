#!/usr/bin/env python3
import json
import resource
import subprocess
import sys
import time
from pathlib import Path

if len(sys.argv) < 6 or sys.argv[4] != "--":
    raise SystemExit(
        "usage: measure-command.py METRICS_JSON STDOUT STDERR -- COMMAND [ARG ...]"
    )

metrics_path = Path(sys.argv[1])
stdout_path = Path(sys.argv[2])
stderr_path = Path(sys.argv[3])
command = sys.argv[5:]
started = time.perf_counter_ns()
with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
    completed = subprocess.run(command, stdout=stdout, stderr=stderr, check=False)
elapsed_ms = (time.perf_counter_ns() - started) // 1_000_000
peak_rss_kib = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
metrics_path.write_text(
    json.dumps(
        {
            "elapsedMs": elapsed_ms,
            "peakRssKiB": peak_rss_kib,
            "exitCode": completed.returncode,
        }
    )
    + "\n",
    encoding="utf-8",
)
raise SystemExit(completed.returncode)
