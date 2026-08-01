#!/usr/bin/env python3
import argparse
import json
import statistics
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate Full V1 qualification evidence against the frozen program."
    )
    subparsers = parser.add_subparsers(dest="gate", required=True)
    reference = subparsers.add_parser("reference")
    reference.add_argument("evidence", type=Path)
    reference.add_argument("--output", type=Path, required=True)
    progress = subparsers.add_parser("progress")
    progress.add_argument("evidence", type=Path)
    progress.add_argument("--output", type=Path, required=True)
    native = subparsers.add_parser("native")
    native.add_argument("evidence", type=Path, help="directory of native smoke JSON records")
    native.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def read_json(path: Path) -> dict:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot read {path}: {error}") from error
    if not isinstance(value, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return value


def number(value: object) -> int | float | None:
    return value if isinstance(value, (int, float)) and not isinstance(value, bool) else None


def validate_reference(evidence: dict, protocol: dict, runner: dict) -> dict:
    failures: list[str] = []
    expected_runner = runner["runnerId"]
    if evidence.get("runnerId") != expected_runner:
        failures.append(f"runnerId must be {expected_runner}")
    image = evidence.get("resolvedImageVersion")
    if not isinstance(image, str) or not image or image == "local-unqualified":
        failures.append("resolvedImageVersion must identify the hosted runner image")
    if evidence.get("logicalProcessors") != runner["cpuAllocation"]["logicalProcessors"]:
        failures.append(
            "logicalProcessors must match the frozen reference-runner allocation"
        )

    workload_fields = {
        "fileCount",
        "lineCount",
        "elapsedMilliseconds",
        "peakRssKiB",
    }
    required_top_level = set(runner["requiredEvidence"]) - workload_fields
    for field in sorted(required_top_level):
        if field not in evidence or evidence[field] in (None, "", "unknown"):
            failures.append(f"missing required evidence field {field}")

    performance = protocol["automatedQualification"]["performance"]
    expected = {workload["id"]: workload for workload in performance["coldWorkloads"]}
    workloads = evidence.get("workloads")
    if not isinstance(workloads, list):
        failures.append("workloads must be an array")
        workloads = []
    observed: dict[str, dict] = {}
    for workload in workloads:
        if isinstance(workload, dict) and isinstance(workload.get("id"), str):
            if workload["id"] in observed:
                failures.append(f"duplicate workload {workload['id']}")
            observed[workload["id"]] = workload
    for workload_id, requirement in expected.items():
        workload = observed.get(workload_id)
        if workload is None:
            failures.append(f"missing workload {workload_id}")
            continue
        file_count = number(workload.get("fileCount"))
        line_count = number(workload.get("lineCount"))
        elapsed = number(workload.get("elapsedMilliseconds"))
        peak_rss = number(workload.get("peakRssKiB"))
        if file_count is None:
            failures.append(f"{workload_id} requires numeric fileCount")
        if line_count is None:
            failures.append(f"{workload_id} requires numeric lineCount")
        if elapsed is None:
            failures.append(f"{workload_id} requires numeric elapsedMilliseconds")
        if peak_rss is None:
            failures.append(f"{workload_id} requires numeric peakRssKiB")
        if "minimumFiles" in requirement and file_count is not None and file_count < requirement["minimumFiles"]:
            failures.append(
                f"{workload_id} must satisfy minimumFiles={requirement['minimumFiles']}"
            )
        if "minimumLines" in requirement and line_count is not None and line_count < requirement["minimumLines"]:
            failures.append(
                f"{workload_id} must satisfy minimumLines={requirement['minimumLines']}"
            )
        if elapsed is not None and elapsed > performance["maximumElapsedSecondsPerWorkload"] * 1000:
            failures.append(f"{workload_id} exceeded the elapsed-time gate")
        if peak_rss is not None and peak_rss * 1024 > performance["maximumPeakMemoryBytesPerWorkload"]:
            failures.append(f"{workload_id} exceeded the peak-memory gate")
        if workload.get("exitCode") != 0:
            failures.append(f"{workload_id} scanner exitCode must be 0")

    return {
        "gate": "reference",
        "status": "pass" if not failures else "fail",
        "failures": failures,
        "runnerId": evidence.get("runnerId"),
        "resolvedImageVersion": evidence.get("resolvedImageVersion"),
        "workloads": sorted(observed),
    }


def validate_progress(evidence: dict, protocol: dict, runner: dict) -> dict:
    failures: list[str] = []
    expected_runner = runner["runnerId"]
    if evidence.get("runnerId") != expected_runner:
        failures.append(f"runnerId must be {expected_runner}")
    image = evidence.get("resolvedImageVersion")
    if not isinstance(image, str) or not image or image == "local-unqualified":
        failures.append("resolvedImageVersion must identify the hosted runner image")
    if evidence.get("logicalProcessors") != runner["cpuAllocation"]["logicalProcessors"]:
        failures.append(
            "logicalProcessors must match the frozen reference-runner allocation"
        )
    for field in (
        "scannerRevision",
        "scannerVersion",
        "rulePackVersion",
        "fixtureVersion",
    ):
        if evidence.get(field) in (None, "", "unknown"):
            failures.append(f"missing required evidence field {field}")

    progress = protocol["automatedQualification"]["progress"]
    required_pairs = progress["alternatingColdPairs"]
    pairs = evidence.get("pairs")
    if not isinstance(pairs, list):
        failures.append("pairs must be an array")
        pairs = []
    if len(pairs) != required_pairs:
        failures.append(f"progress trial must contain {required_pairs} alternating pairs")
    deltas: list[float] = []
    report_hashes: set[str] = set()
    exit_codes: set[int] = set()
    for index, pair in enumerate(pairs):
        if not isinstance(pair, dict):
            failures.append(f"pair {index + 1} must be an object")
            continue
        expected_order = ["always", "never"] if index % 2 == 0 else ["never", "always"]
        if pair.get("pair") != index + 1 or pair.get("order") != expected_order:
            failures.append(f"pair {index + 1} violates the alternating order")
        delta = number(pair.get("deltaPercent"))
        always_ns = number(pair.get("alwaysNs"))
        never_ns = number(pair.get("neverNs"))
        if delta is None:
            failures.append(f"pair {index + 1} has no numeric deltaPercent")
        if always_ns is None or never_ns is None or always_ns < 0 or never_ns <= 0:
            failures.append(f"pair {index + 1} has invalid timing values")
        else:
            recomputed = (always_ns - never_ns) * 100 / never_ns
            deltas.append(float(recomputed))
            if delta is not None and abs(recomputed - delta) > 1e-9:
                failures.append(f"pair {index + 1} recorded delta does not match recomputed delta")
        digest = pair.get("reportSha256")
        if isinstance(digest, str) and len(digest) == 64:
            report_hashes.add(digest)
        else:
            failures.append(f"pair {index + 1} has no valid reportSha256")
        exit_code = pair.get("exitCode")
        if isinstance(exit_code, int):
            exit_codes.add(exit_code)
        else:
            failures.append(f"pair {index + 1} has no exitCode")
    if len(report_hashes) > 1:
        failures.append("progress-on and progress-off report bytes are not equivalent")
    if len(exit_codes) > 1:
        failures.append("progress-on and progress-off exit codes are not equivalent")
    if exit_codes and exit_codes != {0}:
        failures.append("progress trial scanner exitCode must be 0")

    median = statistics.median(deltas) if deltas else None
    maximum_percent = progress["maximumMedianPairedOverhead"] * 100
    if median is None or median > maximum_percent:
        failures.append(f"median paired progress overhead must not exceed {maximum_percent:g}%")
    recorded_median = evidence.get("medianPairedOverheadPercent")
    if median is not None and recorded_median != median:
        failures.append("recorded median does not match the paired distribution")
    interval = evidence.get("empirical95PercentInterval")
    if not (
        isinstance(interval, list)
        and len(interval) == 2
        and all(isinstance(value, (int, float)) for value in interval)
    ):
        failures.append("empirical95PercentInterval must contain two numeric bounds")

    return {
        "gate": "progress",
        "status": "pass" if not failures else "fail",
        "failures": failures,
        "runnerId": evidence.get("runnerId"),
        "resolvedImageVersion": evidence.get("resolvedImageVersion"),
        "pairCount": len(pairs),
        "medianPairedOverheadPercent": median,
        "reportAndOutcomeEquivalent": len(report_hashes) == 1 and len(exit_codes) == 1,
    }


def validate_native(records: list[dict], protocol: dict) -> dict:
    failures: list[str] = []
    expected_targets = protocol["automatedQualification"]["nativeTargets"]
    observed: dict[str, dict] = {}
    for record in records:
        target = record.get("target")
        if not isinstance(target, str) or not target:
            failures.append("native record has no target")
            continue
        if target in observed:
            failures.append(f"duplicate native target {target}")
        observed[target] = record
    for target in expected_targets:
        if target not in observed:
            failures.append(f"missing native target {target}")
    for target in sorted(set(observed) - set(expected_targets)):
        failures.append(f"unexpected native target {target}")

    revisions: set[str] = set()
    versions: set[str] = set()
    rule_packs: set[str] = set()
    for target in expected_targets:
        record = observed.get(target)
        if record is None:
            continue
        for field in (
            "runnerOs",
            "runnerArch",
            "resolvedImageVersion",
            "scannerRevision",
            "scannerVersion",
            "rulePackVersion",
            "binarySha256",
            "binaryBytes",
        ):
            if record.get(field) in (None, "", "unknown", "local-unqualified"):
                failures.append(f"{target} missing qualified {field}")
        binary_digest = record.get("binarySha256")
        if not (
            isinstance(binary_digest, str)
            and len(binary_digest) == 64
            and all(character in "0123456789abcdef" for character in binary_digest)
        ):
            failures.append(f"{target} binarySha256 must be a lowercase SHA-256 digest")
        binary_bytes = record.get("binaryBytes")
        if not isinstance(binary_bytes, int) or isinstance(binary_bytes, bool) or binary_bytes <= 0:
            failures.append(f"{target} binaryBytes must be a positive integer")
        revisions.add(str(record.get("scannerRevision")))
        versions.add(str(record.get("scannerVersion")))
        rule_packs.add(str(record.get("rulePackVersion")))
        if record.get("versionExitCode") != 0:
            failures.append(f"{target} version smoke failed")
        if record.get("scanExitCode") != 0:
            failures.append(f"{target} scan smoke failed")
        digests = record.get("reportSha256")
        if not (
            isinstance(digests, list)
            and len(digests) == 2
            and all(isinstance(digest, str) and len(digest) == 64 for digest in digests)
            and digests[0] == digests[1]
        ):
            failures.append(f"{target} scan output is not deterministic")
    if len(revisions) > 1:
        failures.append("native records do not share one scannerRevision")
    if len(versions) > 1:
        failures.append("native records do not share one scannerVersion")
    if len(rule_packs) > 1:
        failures.append("native records do not share one rulePackVersion")

    return {
        "gate": "native",
        "status": "pass" if not failures else "fail",
        "failures": failures,
        "qualifiedTargets": [target for target in expected_targets if target in observed],
        "scannerRevision": next(iter(revisions)) if len(revisions) == 1 else None,
        "scannerVersion": next(iter(versions)) if len(versions) == 1 else None,
        "rulePackVersion": next(iter(rule_packs)) if len(rule_packs) == 1 else None,
    }


def main() -> int:
    args = parse_args()
    try:
        protocol = read_json(ROOT / "qualification" / "protocol.json")
        runner = read_json(ROOT / "qualification" / "reference-runner.json")
        if args.gate == "reference":
            evidence = read_json(args.evidence)
            decision = validate_reference(evidence, protocol, runner)
        elif args.gate == "progress":
            evidence = read_json(args.evidence)
            decision = validate_progress(evidence, protocol, runner)
        else:
            try:
                paths = sorted(args.evidence.glob("*.json"))
            except OSError as error:
                raise ValueError(f"cannot list {args.evidence}: {error}") from error
            if not paths:
                raise ValueError(f"{args.evidence} contains no native smoke JSON records")
            decision = validate_native([read_json(path) for path in paths], protocol)
    except ValueError as error:
        print(f"qualification program: {error}", file=sys.stderr)
        return 1
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(decision, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"qualification decision: {args.output}")
    for failure in decision["failures"]:
        print(f"FAIL: {failure}", file=sys.stderr)
    return int(decision["status"] != "pass")


if __name__ == "__main__":
    raise SystemExit(main())
