#!/usr/bin/env python3
import argparse
import hashlib
import json
import random
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path


ALLOWED_EXIT_CODES = {0, 2, 3}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run deterministic malformed/static-source smoke fuzzing through the CLI."
    )
    parser.add_argument("--scanner", type=Path, default=Path("target/debug/ai-ui-slop"))
    parser.add_argument("--iterations", type=int, default=128)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument(
        "--output", type=Path, default=Path("target/qualification/fuzz-smoke.json")
    )
    parser.add_argument("--timeout-seconds", type=float, default=10.0)
    return parser.parse_args()


def case_source(index: int, randomizer: random.Random) -> bytes:
    token = "".join(randomizer.choice("abcXYZ0123:-[]_/.") for _ in range(48))
    if index % 2 == 0:
        classes = [
            "rounded-3xl",
            "p-8",
            "dark:shadow-2xl",
            "light:bg-[linear-gradient(red,blue)]",
            f"data-[state={token[:8]}]:border",
        ]
        randomizer.shuffle(classes)
        return (
            f'export function Case{index}(){{return <main className="{" ".join(classes)}">'
            f"{token}</main>}}\n"
        ).encode()
    malformed = bytearray(f"<main className={{runtime_{token}}}>\n".encode())
    malformed.extend([0xFF, 0xFE, index % 256])
    return bytes(malformed)


def digest(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def run_once(scanner: Path, repository: Path, timeout: float) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        [
            str(scanner),
            "scan",
            str(repository),
            "--format",
            "json",
            "--progress",
            "never",
        ],
        check=False,
        capture_output=True,
        timeout=timeout,
    )


def main() -> int:
    args = parse_args()
    if args.iterations < 1:
        print("fuzz smoke: --iterations must be positive", file=sys.stderr)
        return 1
    scanner = args.scanner.resolve()
    if not scanner.is_file():
        print(f"fuzz smoke: scanner does not exist: {scanner}", file=sys.stderr)
        return 1
    randomizer = random.Random(args.seed)
    cases = []
    outcomes: Counter[int] = Counter()
    try:
        with tempfile.TemporaryDirectory(prefix="ai-ui-slop-fuzz-") as directory:
            root = Path(directory)
            for index in range(args.iterations):
                repository = root / f"case-{index:05d}"
                repository.mkdir()
                source = case_source(index, randomizer)
                (repository / "Case.tsx").write_bytes(source)
                first = run_once(scanner, repository, args.timeout_seconds)
                second = run_once(scanner, repository, args.timeout_seconds)
                if first.returncode == 4:
                    raise RuntimeError(f"case {index}: scanner returned internal-error exit 4")
                if first.returncode not in ALLOWED_EXIT_CODES:
                    raise RuntimeError(
                        f"case {index}: unexpected scanner exit {first.returncode}"
                    )
                if (first.returncode, first.stdout, first.stderr) != (
                    second.returncode,
                    second.stdout,
                    second.stderr,
                ):
                    raise RuntimeError(f"case {index}: repeated scan was not deterministic")
                outcomes[first.returncode] += 1
                cases.append(
                    {
                        "case": index,
                        "exitCode": first.returncode,
                        "inputSha256": digest(source),
                        "stderrSha256": digest(first.stderr),
                        "stdoutSha256": digest(first.stdout),
                    }
                )
    except subprocess.TimeoutExpired as error:
        print(f"fuzz smoke: scanner timed out after {error.timeout}s", file=sys.stderr)
        return 1
    except RuntimeError as error:
        print(f"fuzz smoke: {error}", file=sys.stderr)
        return 1

    report = {
        "cases": cases,
        "iterations": args.iterations,
        "outcomes": {str(code): count for code, count in sorted(outcomes.items())},
        "seed": args.seed,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"fuzz smoke evidence: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
