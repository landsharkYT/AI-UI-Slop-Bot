import json
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "mutation-score.py"


class MutationScoreTest(unittest.TestCase):
    def write_outcomes(self, root: Path, caught: int, missed: int, timeout: int, unviable: int) -> None:
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
                    "caught": 8,
                    "minimumScorePercent": 80.0,
                    "missed": 2,
                    "passed": True,
                    "scorePercent": 80.0,
                    "timedOut": 0,
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


if __name__ == "__main__":
    unittest.main()
