import json
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "audit-qualification.py"


class QualificationAuditTest(unittest.TestCase):
    def test_json_summary_counts_pending_and_verified_rows(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs" / "evidence").mkdir(parents=True)
            (root / "docs" / "evidence" / "REQ-002.md").write_text(
                "# REQ-002 evidence\n", encoding="utf-8"
            )
            (root / "docs" / "requirements-verification.md").write_text(
                "\n".join(
                    [
                        "| Requirement | First required milestone | Planned verification | Required evidence record | Requirement summary | Status |",
                        "|---|---|---|---|---|---|",
                        "| `REQ-001` | Full V1 | Test | `docs/evidence/REQ-001.md` | Pending | pending |",
                        "| `REQ-002` | Full V1 | Test | `docs/evidence/REQ-002.md` | Verified | local-pass |",
                    ]
                ),
                encoding="utf-8",
            )

            result = subprocess.run(
                [str(SCRIPT), "--root", str(root), "--format", "json"],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                json.loads(result.stdout),
                {
                    "evidenceFiles": 1,
                    "invalidRows": 0,
                    "requirements": 2,
                    "statuses": {"local-pass": 1, "pending": 1},
                    "unverified": 1,
                },
            )

    def test_verified_row_fails_when_evidence_is_missing_or_does_not_cite_id(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs" / "evidence").mkdir(parents=True)
            (root / "docs" / "evidence" / "REQ-002.md").write_text(
                "wrong evidence\n", encoding="utf-8"
            )
            (root / "docs" / "requirements-verification.md").write_text(
                "\n".join(
                    [
                        "| Requirement | First required milestone | Planned verification | Required evidence record | Requirement summary | Status |",
                        "|---|---|---|---|---|---|",
                        "| `REQ-001` | Full V1 | Test | `docs/evidence/REQ-001.md` | Missing | local-pass |",
                        "| `REQ-002` | Full V1 | Test | `docs/evidence/REQ-002.md` | Wrong | local-pass |",
                    ]
                ),
                encoding="utf-8",
            )

            result = subprocess.run(
                [str(SCRIPT), "--root", str(root), "--format", "json"],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 1)
            self.assertIn("REQ-001: evidence file does not exist", result.stderr)
            self.assertIn("REQ-002: evidence does not cite requirement ID", result.stderr)

    def test_rejects_a_requirement_row_that_cannot_be_parsed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs").mkdir()
            (root / "docs" / "requirements-verification.md").write_text(
                "| `REQ-001` | Full V1 | Test | evidence missing columns |\n",
                encoding="utf-8",
            )

            result = subprocess.run(
                [str(SCRIPT), "--root", str(root)],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 1)
            self.assertIn("REQ-001: matrix row could not be parsed", result.stderr)


if __name__ == "__main__":
    unittest.main()
