---
name: audit-and-fix-ui-slop
description: Audit React frontend repositories with AI UI Slop Bot, interpret coverage and evidence, remediate justified repetitive or context-insensitive UI patterns, and verify the result without blindly optimizing a score. Use when an agent is asked to check a repository for AI-looking or vibe-coded UI convergence, review an AI UI Slop report, fix supported findings, establish a safe advisory scan loop, or distinguish intentional House Style from defects and narrow suppressions.
---

# Audit and fix UI slop

Use the bundled runner to produce durable scan artifacts, then apply human-readable judgment to exact evidence. Treat the detector as a review assistant, not a taste oracle.

## Workflow

1. Inspect the target repository, its instructions, current Git status, framework, tests, and design system. Preserve unrelated changes.
2. Run `scripts/ai-ui-slop-agent.sh doctor <repo>`. Set `AI_UI_SLOP_BIN` when `ai-ui-slop` is not on `PATH`.
3. If configuration is absent and the user authorized repository changes, run `scripts/ai-ui-slop-agent.sh init <repo>`. Review the generated scopes and House Style assumptions before scanning. Never approve House Style automatically.
4. Run `scripts/ai-ui-slop-agent.sh scan <repo>`. The runner saves JSON, the Refactoring Brief, stderr, validation output, version, and exit code under `.ai-ui-slop/agent-runs/`.
5. Read `report.json` first and `refactoring-brief.md` second. Read [references/remediation.md](references/remediation.md) before choosing fixes.
6. Verify each candidate against its cited source, resolved styles, component role, reachable state, and coverage. Do not infer a clean result from zero findings when coverage is limited or the scope is `not_applicable`.
7. Classify each candidate as one of: fix, intentional House Style, narrow suppression, or unresolved/advisory. Ask for Design Authority input when the choice would encode product taste or broaden policy.
8. For justified fixes, preserve behavior, accessibility, responsive behavior, focus order, content, and user workflows. Prefer role-specific hierarchy and product-specific structure over cosmetic novelty. Add or update tests when the repository has an applicable test surface.
9. Run the repository's own formatter, linter, type checker, tests, and build. Re-run the bundled scan command.
10. Compare exact findings and coverage before and after. Do not claim success merely because the aggregate score fell; explain which evidence was removed, retained, approved, suppressed, or made unresolved.

## Guardrails

- Keep scans advisory unless the repository already has a reviewed compatible baseline and the user explicitly asks for enforcement.
- Never run `baseline accept`, enable enforcement, add broad suppressions, or rewrite House Style without explicit Design Authority approval.
- Never execute target-repository JavaScript or configuration merely to improve scanner coverage.
- Treat scanner exit `3` as insufficient analysis or artifact coverage, not as “no issues.” Preserve all exit codes.
- Do not fix unsupported or ambiguous evidence by guessing. Report the limitation and the smallest useful next step.
- Do not redesign an entire product from a single finding unless the user requested that scope.
- Do not remove purposeful repeated components solely to reduce recurrence.

## Runner commands

```sh
# Read-only preflight
scripts/ai-ui-slop-agent.sh doctor .

# Writes ai-ui-slop.config.jsonc only when absent
scripts/ai-ui-slop-agent.sh init .

# Validates config, scans, and records an agent-readable run directory
scripts/ai-ui-slop-agent.sh scan .
```

The runner itself does not edit application source, accept baselines, or apply fixes. It prints the durable run directory and returns the scanner's exit code. Read the human installation notes in [README.md](README.md) only when installing or copying the skill.

## Completion report

Report:

- scanner, rule-pack, and report-schema versions;
- applicability and material coverage limitations;
- findings reviewed and disposition of each;
- source and test files changed;
- repository verification commands and outcomes;
- before/after finding evidence and score, without presenting the score as objective quality; and
- any decisions still requiring Design Authority review.
