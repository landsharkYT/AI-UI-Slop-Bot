import json
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "qualification-program.py"


def reference_evidence() -> dict:
    return {
        "protocolVersion": "1",
        "runnerId": "github-ubuntu-24.04-x64-v1",
        "resolvedImageVersion": "20260720.1.0",
        "osRelease": "Ubuntu 24.04.3 LTS",
        "kernel": "6.11.0",
        "cpuModel": "AMD EPYC 7763",
        "logicalProcessors": 4,
        "memoryBytes": 16_000_000_000,
        "rustcVersion": "rustc 1.96.0",
        "cargoVersion": "cargo 1.96.0",
        "scannerRevision": "abc123",
        "scannerVersion": "ai-ui-slop 0.13.0",
        "rulePackVersion": "1.0.0-beta.7",
        "scannerOptions": ["scan", "<fixture>", "--format", "json", "--progress", "never"],
        "fixtureVersion": "2",
        "workloads": [
            {
                "id": "representative-files",
                "fileCount": 2000,
                "lineCount": 2000,
                "elapsedMilliseconds": 900,
                "peakRssKiB": 20_000,
                "exitCode": 0,
                "passesElapsedGate": True,
                "passesMemoryGate": True,
            },
            {
                "id": "representative-lines",
                "fileCount": 500,
                "lineCount": 500000,
                "elapsedMilliseconds": 1200,
                "peakRssKiB": 21_000,
                "exitCode": 0,
                "passesElapsedGate": True,
                "passesMemoryGate": True,
            },
        ],
    }


def progress_evidence() -> dict:
    return {
        "protocolVersion": "1",
        "runnerId": "github-ubuntu-24.04-x64-v1",
        "resolvedImageVersion": "20260720.1.0",
        "logicalProcessors": 4,
        "scannerRevision": "abc123",
        "scannerVersion": "ai-ui-slop 0.13.0",
        "rulePackVersion": "1.0.0-beta.7",
        "fixtureVersion": "2",
        "pairs": [
            {
                "pair": index + 1,
                "order": ["always", "never"] if index % 2 == 0 else ["never", "always"],
                "alwaysNs": 101_000_000,
                "neverNs": 100_000_000,
                "deltaPercent": 1.0,
                "reportSha256": "a" * 64,
                "exitCode": 0,
            }
            for index in range(20)
        ],
        "medianPairedOverheadPercent": 1.0,
        "empirical95PercentInterval": [1.0, 1.0],
        "passesTwoPercentMedianGate": True,
    }


class QualificationProgramTest(unittest.TestCase):
    def run_gate(self, gate: str, evidence: dict) -> tuple[subprocess.CompletedProcess, dict]:
        with tempfile.TemporaryDirectory() as directory:
            temporary = Path(directory)
            evidence_path = temporary / "evidence.json"
            output_path = temporary / "decision.json"
            evidence_path.write_text(json.dumps(evidence), encoding="utf-8")
            result = subprocess.run(
                [str(SCRIPT), gate, str(evidence_path), "--output", str(output_path)],
                check=False,
                capture_output=True,
                text=True,
            )
            decision = (
                json.loads(output_path.read_text(encoding="utf-8"))
                if output_path.is_file()
                else {}
            )
            return result, decision

    def run_native(self, records: list[dict]) -> tuple[subprocess.CompletedProcess, dict]:
        with tempfile.TemporaryDirectory() as directory:
            temporary = Path(directory)
            evidence_directory = temporary / "native"
            evidence_directory.mkdir()
            for index, record in enumerate(records):
                (evidence_directory / f"record-{index}.json").write_text(
                    json.dumps(record), encoding="utf-8"
                )
            output_path = temporary / "decision.json"
            result = subprocess.run(
                [
                    str(SCRIPT),
                    "native",
                    str(evidence_directory),
                    "--output",
                    str(output_path),
                ],
                check=False,
                capture_output=True,
                text=True,
            )
            decision = (
                json.loads(output_path.read_text(encoding="utf-8"))
                if output_path.is_file()
                else {}
            )
            return result, decision

    def test_reference_gate_accepts_complete_pinned_runner_evidence(self) -> None:
        result, decision = self.run_gate("reference", reference_evidence())

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(decision["gate"], "reference")
        self.assertEqual(decision["status"], "pass")
        self.assertEqual(decision["runnerId"], "github-ubuntu-24.04-x64-v1")
        self.assertEqual(decision["workloads"], ["representative-files", "representative-lines"])

    def test_reference_gate_fails_closed_for_local_or_undersized_evidence(self) -> None:
        evidence = reference_evidence()
        evidence["runnerId"] = "local-unqualified"
        evidence["resolvedImageVersion"] = "local-unqualified"
        evidence["workloads"][0]["fileCount"] = 1999

        result, decision = self.run_gate("reference", evidence)

        self.assertEqual(result.returncode, 1)
        self.assertEqual(decision["status"], "fail")
        self.assertIn("runnerId", " ".join(decision["failures"]))
        self.assertIn("minimumFiles=2000", " ".join(decision["failures"]))

    def test_reference_gate_reports_malformed_metrics_without_crashing(self) -> None:
        evidence = reference_evidence()
        evidence["workloads"][0]["elapsedMilliseconds"] = "fast"
        evidence["workloads"][1]["peakRssKiB"] = None

        result, decision = self.run_gate("reference", evidence)

        self.assertEqual(result.returncode, 1)
        self.assertNotIn("Traceback", result.stderr)
        self.assertIn("numeric elapsedMilliseconds", " ".join(decision["failures"]))
        self.assertIn("numeric peakRssKiB", " ".join(decision["failures"]))

    def test_progress_gate_recomputes_pairing_equivalence_and_thresholds(self) -> None:
        result, decision = self.run_gate("progress", progress_evidence())

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(decision["status"], "pass")
        self.assertEqual(decision["pairCount"], 20)
        self.assertEqual(decision["medianPairedOverheadPercent"], 1.0)

    def test_progress_gate_rejects_local_partial_or_behavior_changing_trials(self) -> None:
        evidence = progress_evidence()
        evidence["runnerId"] = "local-unqualified"
        evidence["pairs"].pop()
        evidence["pairs"][1]["reportSha256"] = "b" * 64
        for pair in evidence["pairs"]:
            pair["alwaysNs"] = 102_100_000
            pair["neverNs"] = 100_000_000
            pair["deltaPercent"] = 2.1
        evidence["medianPairedOverheadPercent"] = 2.1

        result, decision = self.run_gate("progress", evidence)

        self.assertEqual(result.returncode, 1)
        failures = " ".join(decision["failures"])
        self.assertIn("runnerId", failures)
        self.assertIn("20 alternating", failures)
        self.assertIn("report bytes", failures)
        self.assertIn("2%", failures)

    def test_progress_gate_recomputes_each_delta_and_requires_successful_scans(self) -> None:
        evidence = progress_evidence()
        evidence["pairs"][0]["alwaysNs"] = 150_000_000
        evidence["pairs"][0]["deltaPercent"] = 1.0
        for pair in evidence["pairs"]:
            pair["exitCode"] = 4

        result, decision = self.run_gate("progress", evidence)

        self.assertEqual(result.returncode, 1)
        failures = " ".join(decision["failures"])
        self.assertIn("recomputed delta", failures)
        self.assertIn("scanner exitCode must be 0", failures)

    def test_native_gate_requires_every_target_and_deterministic_smoke_result(self) -> None:
        targets = [
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
            "x86_64-pc-windows-msvc",
        ]
        records = [
            {
                "schemaVersion": "1",
                "target": target,
                "runnerOs": "Linux",
                "runnerArch": "x86_64",
                "resolvedImageVersion": "20260720.1.0",
                "scannerRevision": "abc123",
                "scannerVersion": "ai-ui-slop 0.13.0",
                "rulePackVersion": "1.0.0-beta.7",
                "binarySha256": str(index) * 64,
                "binaryBytes": 10_000_000,
                "versionExitCode": 0,
                "scanExitCode": 0,
                "reportSha256": ["a" * 64, "a" * 64],
            }
            for index, target in enumerate(targets, start=1)
        ]

        result, decision = self.run_native(records)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(decision["status"], "pass")
        self.assertEqual(decision["qualifiedTargets"], targets)

        records.pop()
        records[0]["reportSha256"][1] = "b" * 64
        result, decision = self.run_native(records)

        self.assertEqual(result.returncode, 1)
        failures = " ".join(decision["failures"])
        self.assertIn("missing native target", failures)
        self.assertIn("deterministic", failures)

        records[0]["binarySha256"] = "not-a-digest"
        records[0]["binaryBytes"] = 0
        result, decision = self.run_native(records)
        self.assertEqual(result.returncode, 1)
        failures = " ".join(decision["failures"])
        self.assertIn("binarySha256", failures)
        self.assertIn("binaryBytes", failures)


if __name__ == "__main__":
    unittest.main()
