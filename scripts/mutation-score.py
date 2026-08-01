#!/usr/bin/env python3
import argparse
import json
import sys
from pathlib import Path


OUTCOMES = ("caught", "missed", "timeout", "unviable")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Normalize cargo-mutants text outcomes and enforce the V1 score gate."
    )
    parser.add_argument("directory", type=Path, help="cargo-mutants output directory")
    parser.add_argument("--minimum", type=float, default=80.0)
    parser.add_argument("--output", type=Path)
    return parser.parse_args()


def count_records(path: Path) -> int:
    try:
        return sum(1 for line in path.read_text(encoding="utf-8").splitlines() if line.strip())
    except OSError as error:
        raise RuntimeError(f"cannot read {path}: {error}") from error


def main() -> int:
    args = parse_args()
    if not 0 <= args.minimum <= 100:
        print("mutation score: --minimum must be between 0 and 100", file=sys.stderr)
        return 1
    try:
        counts = {
            outcome: count_records(args.directory / f"{outcome}.txt")
            for outcome in OUTCOMES
        }
    except RuntimeError as error:
        print(f"mutation score: {error}", file=sys.stderr)
        return 1

    viable = counts["caught"] + counts["missed"] + counts["timeout"]
    if viable == 0:
        print("mutation score: no viable mutants were tested", file=sys.stderr)
        return 1
    score = 100.0 * counts["caught"] / viable
    passed = score >= args.minimum
    report = {
        "caught": counts["caught"],
        "minimumScorePercent": args.minimum,
        "missed": counts["missed"],
        "passed": passed,
        "scorePercent": round(score, 4),
        "timedOut": counts["timeout"],
        "unviable": counts["unviable"],
        "viable": viable,
    }
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(
            json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        print(f"mutation evidence: {args.output}")
    else:
        print(json.dumps(report, sort_keys=True, separators=(",", ":")))
    if not passed:
        print(
            f"mutation score: {score:.2f}% is below {args.minimum:.2f}%",
            file=sys.stderr,
        )
    return int(not passed)


if __name__ == "__main__":
    raise SystemExit(main())
