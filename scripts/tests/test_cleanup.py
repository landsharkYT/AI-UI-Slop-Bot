import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SOURCE_CLEANUP = ROOT / "scripts/cleanup.sh"


class CleanupCommandTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.repository = Path(self.temporary.name) / "repository"
        self.repository.mkdir()
        (self.repository / ".git").mkdir()
        (self.repository / "Cargo.toml").write_text(
            '[package]\nname = "cleanup-fixture"\nversion = "0.1.0"\nedition = "2024"\n',
            encoding="utf-8",
        )
        (self.repository / "src").mkdir()
        (self.repository / "src/main.rs").write_text("fn main() {}\n", encoding="utf-8")
        (self.repository / "scripts").mkdir()
        self.cleanup = self.repository / "scripts/cleanup.sh"
        shutil.copy2(SOURCE_CLEANUP, self.cleanup)
        self.populate_generated_files()

    def tearDown(self):
        self.temporary.cleanup()

    def populate_generated_files(self):
        for relative in [
            "target/debug/deps/test-binary",
            "target/release/ai-ui-slop",
            "target/qualification/mutants/outcomes.json",
            "target/mutants-tmp-run/worktree",
            "mutants.out/outcomes.json",
            "mutants.out.old/outcomes.json",
            "mutants.out_backup/outcomes.json",
            ".opencode/node_modules/package/index.js",
            ".opencode/settings.json",
            ".ai-ui-slop/reports/report.json",
            "docs/evidence/TEST-004.md",
        ]:
            path = self.repository / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(relative + "\n", encoding="utf-8")

    def run_cleanup(self, *arguments, environment=None):
        return subprocess.run(
            [str(self.cleanup), *arguments],
            cwd=self.repository,
            env=os.environ.copy() | (environment or {}),
            text=True,
            capture_output=True,
            check=False,
        )

    def test_default_inspection_is_read_only_and_reports_generated_categories(self):
        result = self.run_cleanup()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("mode: inspect", result.stdout)
        self.assertIn("target/debug", result.stdout)
        self.assertIn("target/qualification", result.stdout)
        self.assertIn(".opencode/node_modules", result.stdout)
        self.assertTrue((self.repository / "target/debug/deps/test-binary").exists())
        self.assertTrue((self.repository / "target/release/ai-ui-slop").exists())

    def test_routine_removes_rebuildable_work_and_preserves_release_and_raw_evidence(self):
        result = self.run_cleanup("routine")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse((self.repository / "target/debug").exists())
        self.assertFalse((self.repository / "target/mutants-tmp-run").exists())
        self.assertTrue((self.repository / "target/qualification").exists())
        self.assertTrue((self.repository / "mutants.out").exists())
        self.assertTrue((self.repository / "mutants.out.old").exists())
        self.assertTrue((self.repository / "mutants.out_backup").exists())
        self.assertTrue((self.repository / "target/release/ai-ui-slop").exists())
        self.assertTrue((self.repository / ".opencode/node_modules/package/index.js").exists())
        self.assertTrue((self.repository / ".ai-ui-slop/reports/report.json").exists())
        self.assertTrue((self.repository / "docs/evidence/TEST-004.md").exists())
        self.assertIn("preserved: target/release", result.stdout)
        self.assertIn("preserved: target/qualification and mutants.out*", result.stdout)

    def test_raw_qualification_cleanup_requires_confirmation(self):
        refused = self.run_cleanup("routine", "--include-qualification")
        self.assertEqual(refused.returncode, 2)
        self.assertIn("--yes", refused.stderr)
        self.assertTrue((self.repository / "target/qualification").exists())
        self.assertTrue((self.repository / "mutants.out").exists())

        accepted = self.run_cleanup("routine", "--include-qualification", "--yes")
        self.assertEqual(accepted.returncode, 0, accepted.stderr)
        self.assertFalse((self.repository / "target/qualification").exists())
        self.assertFalse((self.repository / "mutants.out").exists())
        self.assertFalse((self.repository / "mutants.out.old").exists())
        self.assertFalse((self.repository / "mutants.out_backup").exists())
        self.assertTrue((self.repository / "docs/evidence/TEST-004.md").exists())

    def test_dry_run_describes_routine_cleanup_without_deleting(self):
        result = self.run_cleanup("routine", "--dry-run")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("would remove", result.stdout)
        self.assertTrue((self.repository / "target/debug/deps/test-binary").exists())
        self.assertTrue((self.repository / "target/qualification/mutants/outcomes.json").exists())
        self.assertTrue((self.repository / "mutants.out/outcomes.json").exists())

    def test_optional_opencode_cache_requires_confirmation_and_preserves_settings(self):
        refused = self.run_cleanup("routine", "--include-opencode")
        self.assertEqual(refused.returncode, 2)
        self.assertIn("--yes", refused.stderr)
        self.assertTrue((self.repository / ".opencode/node_modules").exists())

        accepted = self.run_cleanup("routine", "--include-opencode", "--yes")
        self.assertEqual(accepted.returncode, 0, accepted.stderr)
        self.assertFalse((self.repository / ".opencode/node_modules").exists())
        self.assertTrue((self.repository / ".opencode/settings.json").exists())

    def test_full_cleanup_requires_confirmation_and_delegates_to_cargo_clean(self):
        refused = self.run_cleanup("full")
        self.assertEqual(refused.returncode, 2)
        self.assertIn("--yes", refused.stderr)
        self.assertTrue((self.repository / "target/release/ai-ui-slop").exists())

        accepted = self.run_cleanup("full", "--yes")
        self.assertEqual(accepted.returncode, 0, accepted.stderr)
        self.assertFalse((self.repository / "target").exists())
        self.assertTrue((self.repository / ".ai-ui-slop/reports/report.json").exists())

    def test_cleanup_refuses_a_symlinked_target_directory(self):
        external = Path(self.temporary.name) / "external"
        external.mkdir()
        marker = external / "keep.txt"
        marker.write_text("keep\n", encoding="utf-8")
        target = self.repository / "target"
        shutil.rmtree(target)
        target.symlink_to(external, target_is_directory=True)

        result = self.run_cleanup("routine")
        self.assertEqual(result.returncode, 2)
        self.assertIn("symbolic link", result.stderr)
        self.assertEqual(marker.read_text(encoding="utf-8"), "keep\n")

    def test_cleanup_refuses_an_opencode_cache_through_a_symlinked_parent(self):
        shutil.rmtree(self.repository / ".opencode")
        external = Path(self.temporary.name) / "external-opencode"
        modules = external / "node_modules"
        modules.mkdir(parents=True)
        marker = modules / "keep.txt"
        marker.write_text("keep\n", encoding="utf-8")
        (self.repository / ".opencode").symlink_to(external, target_is_directory=True)

        result = self.run_cleanup("routine", "--include-opencode", "--yes")
        self.assertEqual(result.returncode, 2)
        self.assertIn("symbolic link", result.stderr)
        self.assertEqual(marker.read_text(encoding="utf-8"), "keep\n")

    def test_command_failures_are_propagated_without_false_success_messages(self):
        fake_bin = Path(self.temporary.name) / "fake-bin"
        fake_bin.mkdir()
        fake_cargo = fake_bin / "cargo"
        fake_cargo.write_text("#!/usr/bin/env bash\nexit 9\n", encoding="utf-8")
        fake_cargo.chmod(0o755)
        environment = {"PATH": f"{fake_bin}:{os.environ['PATH']}"}

        result = self.run_cleanup("full", "--yes", environment=environment)
        self.assertEqual(result.returncode, 9)
        self.assertNotIn("preserved:", result.stdout)
        self.assertTrue((self.repository / "target/release/ai-ui-slop").exists())


class CleanupDocumentationTests(unittest.TestCase):
    def test_readme_documents_safe_routine_full_and_optional_cache_modes(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        for command in [
            "scripts/cleanup.sh inspect",
            "scripts/cleanup.sh routine --dry-run",
            "scripts/cleanup.sh routine",
            "scripts/cleanup.sh full --dry-run",
            "scripts/cleanup.sh full --yes",
            "scripts/cleanup.sh routine --include-opencode --yes",
            "scripts/cleanup.sh routine --include-qualification --yes",
        ]:
            self.assertIn(command, readme)
        self.assertIn("preserving `target/release`", readme)
        self.assertIn("reject a symlinked `target` directory", readme)


if __name__ == "__main__":
    unittest.main()
