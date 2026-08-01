import json
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "native-smoke.py"


def executable(path: Path, body: str) -> None:
    path.write_text(f"#!/usr/bin/env python3\n{body}", encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR)


class NativeSmokeTest(unittest.TestCase):
    def test_records_version_binary_identity_and_two_deterministic_scans(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            temporary = Path(directory)
            scanner = temporary / "scanner"
            output = temporary / "smoke.json"
            executable(
                scanner,
                "import sys\n"
                "if sys.argv[1] == 'version':\n"
                " print('ai-ui-slop 0.13.0')\n"
                " print('rule-pack 1.0.0-beta.7')\n"
                "else:\n"
                " print('{\"schemaVersion\":\"7\",\"summary\":{\"outcome\":\"success\"}}')\n",
            )

            result = subprocess.run(
                [
                    str(SCRIPT),
                    str(scanner),
                    "x86_64-unknown-linux-gnu",
                    str(output),
                ],
                check=False,
                capture_output=True,
                text=True,
                env={"ImageVersion": "20260720.1.0"},
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            evidence = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(evidence["target"], "x86_64-unknown-linux-gnu")
            self.assertEqual(evidence["scannerVersion"], "ai-ui-slop 0.13.0")
            self.assertEqual(evidence["rulePackVersion"], "1.0.0-beta.7")
            self.assertEqual(evidence["versionExitCode"], 0)
            self.assertEqual(evidence["scanExitCode"], 0)
            self.assertEqual(len(evidence["reportSha256"]), 2)
            self.assertEqual(evidence["reportSha256"][0], evidence["reportSha256"][1])
            self.assertEqual(evidence["resolvedImageVersion"], "20260720.1.0")
            self.assertGreater(evidence["binaryBytes"], 0)
            self.assertEqual(len(evidence["binarySha256"]), 64)

    def test_rejects_a_binary_that_cannot_complete_the_scan_smoke(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            temporary = Path(directory)
            scanner = temporary / "scanner"
            executable(scanner, "import sys\nraise SystemExit(0 if sys.argv[1] == 'version' else 4)\n")

            result = subprocess.run(
                [str(SCRIPT), str(scanner), "x86_64-unknown-linux-gnu", str(temporary / "out.json")],
                check=False,
                capture_output=True,
                text=True,
                env={"ImageVersion": "20260720.1.0"},
            )

            self.assertEqual(result.returncode, 1)
            self.assertIn("scan smoke failed", result.stderr)


if __name__ == "__main__":
    unittest.main()
