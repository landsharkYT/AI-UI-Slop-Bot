import json
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "fuzz-smoke.py"


def executable(path: Path, body: str) -> None:
    path.write_text(f"#!/usr/bin/env python3\n{body}", encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR)


class FuzzSmokeTest(unittest.TestCase):
    def test_emits_reproducible_evidence_for_allowed_scanner_outcomes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scanner = root / "scanner"
            evidence = root / "evidence.json"
            executable(
                scanner,
                "import hashlib, pathlib, sys\n"
                "source = next(pathlib.Path(sys.argv[2]).glob('*.tsx')).read_bytes()\n"
                "print(hashlib.sha256(source).hexdigest())\n"
                "raise SystemExit(0 if source.startswith(b'export') else 3)\n",
            )

            result = subprocess.run(
                [
                    str(SCRIPT),
                    "--scanner",
                    str(scanner),
                    "--iterations",
                    "4",
                    "--seed",
                    "17",
                    "--output",
                    str(evidence),
                ],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads(evidence.read_text(encoding="utf-8"))
            self.assertEqual(report["iterations"], 4)
            self.assertEqual(report["seed"], 17)
            self.assertEqual(report["outcomes"], {"0": 2, "3": 2})
            self.assertEqual(len(report["cases"]), 4)

    def test_rejects_internal_error_exit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scanner = root / "scanner"
            executable(scanner, "raise SystemExit(4)\n")

            result = subprocess.run(
                [str(SCRIPT), "--scanner", str(scanner), "--iterations", "1"],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 1)
            self.assertIn("scanner returned internal-error exit 4", result.stderr)


if __name__ == "__main__":
    unittest.main()
