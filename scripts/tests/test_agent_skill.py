import os
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
RUNNER = ROOT / "skills/audit-and-fix-ui-slop/scripts/ai-ui-slop-agent.sh"


class AgentSkillRunnerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.repository = self.root / "repository"
        self.repository.mkdir()
        self.scanner = self.root / "fake-ai-ui-slop"
        self.scanner.write_text(
            textwrap.dedent(
                """\
                #!/usr/bin/env bash
                set -u
                case "${1:-}" in
                  version)
                    printf '%s\\n' 'ai-ui-slop 0.14.0 report-schema 8 rule-pack 1.0.0-beta.8'
                    ;;
                  init)
                    printf '%s\\n' '{"schemaVersion":"1"}' > "$2/ai-ui-slop.config.jsonc"
                    ;;
                  config)
                    test "${2:-}" = validate
                    test -f "$3/ai-ui-slop.config.jsonc"
                    ;;
                  scan)
                    repo=$2
                    mkdir -p "$repo/.ai-ui-slop/reports"
                    printf '%s\\n' '{"schemaVersion":"8","scopes":[]}' > "$repo/.ai-ui-slop/reports/report.json"
                    printf '%s\\n' '# Refactoring brief' > "$repo/.ai-ui-slop/reports/refactoring-brief.md"
                    printf '%s\\n' '{"schemaVersion":"8","scopes":[]}'
                    printf '%s\\n' 'scan progress' >&2
                    exit "${FAKE_SCAN_EXIT:-0}"
                    ;;
                  *) exit 2 ;;
                esac
                """
            ),
            encoding="utf-8",
        )
        self.scanner.chmod(0o755)
        self.environment = os.environ.copy()
        self.environment["AI_UI_SLOP_BIN"] = str(self.scanner)

    def tearDown(self):
        self.temporary.cleanup()

    def run_runner(self, *arguments, **environment):
        env = self.environment | environment
        return subprocess.run(
            [str(RUNNER), *arguments],
            cwd=ROOT,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_doctor_is_read_only_and_reports_missing_configuration(self):
        result = self.run_runner("doctor", str(self.repository))
        self.assertEqual(result.returncode, 2)
        self.assertIn("ai-ui-slop 0.14.0", result.stdout)
        self.assertIn("configuration: absent", result.stdout)
        self.assertFalse((self.repository / "ai-ui-slop.config.jsonc").exists())

    def test_init_refuses_to_replace_existing_configuration(self):
        first = self.run_runner("init", str(self.repository))
        self.assertEqual(first.returncode, 0, first.stderr)
        configuration = self.repository / "ai-ui-slop.config.jsonc"
        original = configuration.read_text(encoding="utf-8")

        second = self.run_runner("init", str(self.repository))
        self.assertEqual(second.returncode, 2)
        self.assertIn("refusing to overwrite", second.stderr)
        self.assertEqual(configuration.read_text(encoding="utf-8"), original)

    def test_scan_preserves_coverage_exit_and_copies_canonical_artifacts(self):
        self.assertEqual(self.run_runner("init", str(self.repository)).returncode, 0)
        result = self.run_runner("scan", str(self.repository), FAKE_SCAN_EXIT="3")
        self.assertEqual(result.returncode, 3, result.stderr)
        run_line = next(
            line for line in result.stdout.splitlines() if line.startswith("agent audit run: ")
        )
        run_directory = Path(run_line.removeprefix("agent audit run: "))
        self.assertEqual((run_directory / "exit-code.txt").read_text().strip(), "3")
        self.assertEqual(
            (run_directory / "config-validation-exit-code.txt").read_text().strip(),
            "0",
        )
        self.assertTrue((run_directory / "report.json").is_file())
        self.assertTrue((run_directory / "refactoring-brief.md").is_file())
        self.assertIn("scan progress", (run_directory / "scan.stderr").read_text())


if __name__ == "__main__":
    unittest.main()
