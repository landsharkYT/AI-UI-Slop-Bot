import json
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "mutation-score.py"


class MutationScoreTest(unittest.TestCase):
    def write_outcomes(
        self,
        root: Path,
        caught: int,
        missed: int,
        timeout: int,
        unviable: int,
        *,
        complete: bool = True,
    ) -> None:
        for name, count in {
            "caught": caught,
            "missed": missed,
            "timeout": timeout,
            "unviable": unviable,
        }.items():
            (root / f"{name}.txt").write_text(
                "".join(f"{name} mutant {index}\n" for index in range(count)),
                encoding="utf-8",
            )
        tested = caught + missed + timeout + unviable
        (root / "outcomes.json").write_text(
            json.dumps(
                {
                    "cargo_mutants_version": "27.0.0",
                    "end_time": "2026-07-31T00:00:00Z" if complete else None,
                    "outcomes": [{} for _ in range(tested + 1)],
                    "total_mutants": tested if complete else tested + 1,
                }
            ),
            encoding="utf-8",
        )

    def test_writes_normalized_passing_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence = root / "evidence.json"
            self.write_outcomes(root, caught=8, missed=2, timeout=0, unviable=3)

            result = subprocess.run(
                [str(SCRIPT), str(root), "--output", str(evidence)],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                json.loads(evidence.read_text(encoding="utf-8")),
                {
                    "cargoMutantsVersion": "27.0.0",
                    "caught": 8,
                    "minimumScorePercent": 80.0,
                    "missed": 2,
                    "passed": True,
                    "scorePercent": 80.0,
                    "timedOut": 0,
                    "testedMutants": 13,
                    "totalMutants": 13,
                    "unviable": 3,
                    "viable": 10,
                },
            )

    def test_timeout_counts_as_not_caught_and_fails_threshold(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_outcomes(root, caught=7, missed=1, timeout=2, unviable=0)

            result = subprocess.run(
                [str(SCRIPT), str(root)],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 1)
            self.assertIn("70.00% is below 80.00%", result.stderr)

    def test_refuses_empty_viable_set(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_outcomes(root, caught=0, missed=0, timeout=0, unviable=2)

            result = subprocess.run(
                [str(SCRIPT), str(root)],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 1)
            self.assertIn("no viable mutants", result.stderr)

    def test_resolves_nested_mutants_out_and_rejects_incomplete_run(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            nested = root / "mutants.out"
            nested.mkdir()
            self.write_outcomes(
                nested, caught=8, missed=1, timeout=0, unviable=0, complete=False
            )

            result = subprocess.run(
                [str(SCRIPT), str(root)],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 1)
            self.assertIn("incomplete cargo-mutants run: tested 9 of 10", result.stderr)

    def test_rejects_iterative_output_as_release_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_outcomes(root, caught=8, missed=2, timeout=0, unviable=0)
            (root / "previously_caught.txt").write_text(
                "a result accumulated by --iterate\n", encoding="utf-8"
            )

            result = subprocess.run(
                [str(SCRIPT), str(root)],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 1)
            self.assertIn("iterative cargo-mutants output", result.stderr)


if __name__ == "__main__":
    unittest.main()
