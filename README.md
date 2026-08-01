# AI UI Slop Bot

AI UI Slop Bot is an evidence-first Rust analyzer for repeated, context-insensitive frontend aesthetics: interchangeable landing-page formulas, excessive card shells, stock framework recipes, effect stacking, and the “everything has the same compact border” treatment.

The current build is a **V1 Implementation Candidate** (`0.14.0`, rule pack `1.0.0-beta.8`). It is useful for design-review trials, not an objective taste oracle. The bounded V1 feature surface and provisional mutation gate are locally qualified; customer calibration, hosted-platform qualification, reference-runner performance, and authenticated release validation remain before Full V1. See the [support matrix](docs/support-matrix.md).

Advisory use-case testing may begin now. Keep enforcement disabled until the applicable Full V1 gates pass, record scanner and rule-pack versions, and preserve every reported coverage limitation.

Upgrading an existing checkout requires a reviewed semantic migration. See [Migrating from 0.13 to 0.14](docs/migrations/0.13-to-0.14.md) before replacing reports or a Reviewed Baseline.

## Build

Rust 1.96 or newer is the currently verified toolchain.

```sh
cargo build --release
./target/release/ai-ui-slop version
```

The examples below use `ai-ui-slop` for readability. Until packaged binaries are published, substitute `./target/release/ai-ui-slop`.

## Recommended first audit

Start in advisory mode and review the generated assumptions before trusting the result:

```sh
ai-ui-slop init ./your-react-repository
ai-ui-slop config validate ./your-react-repository
ai-ui-slop config validate ./your-react-repository --effective default
ai-ui-slop scan ./your-react-repository --format terminal --progress always
```

`init` creates `ai-ui-slop.config.jsonc` and refuses to overwrite an existing file. Check these fields in particular:

- `scopes`: each browser application should be its own Analysis Scope; remove server, contract, generated, and unrelated packages.
- `houseStyle.intent`: briefly state what the product should feel like.
- `houseStyle.approvedSignals`, `approvedValues`, and `approvedPrimitives`: record deliberate, reviewed design-system choices.
- `classFunctions` and `componentWrappers`: add repository-specific static class combinators and transparent React wrappers.
- `tailwindVersion`: leave `auto` only when the manifest, lockfile, or configuration identifies it correctly.
- `resources`: keep the defaults initially; lower them only when a bounded CI environment requires it.

The scan always writes the canonical artifacts to:

```text
.ai-ui-slop/reports/report.json
.ai-ui-slop/reports/refactoring-brief.md
```

Use `--format terminal` for triage, `--format markdown` for a human-readable stdout projection, or `--format json` for automation. Progress and diagnostics go to stderr, so JSON stdout remains parseable. `--progress auto` is the default; use `always` to show phase bars in redirected/CI output and `never` for silent stderr.

## How to interpret a result

Do not use the repository score alone as a quality grade. Read the result in this order:

1. **Applicability and coverage:** `not_applicable` means the scope had no supported React source input and is not a clean-UI result. Otherwise, `complete` or `partial` dimensions tell you what the scanner could resolve; a `coverage_limited` interpretation cannot support a clean bill of health.
2. **Finding evidence:** verify that each signal-specific snippet actually supports the claimed pattern.
3. **Owner and reachable state:** confirm the finding belongs to the page/component and state you care about.
4. **Rule explanation:** decide whether the repeated treatment is context-insensitive or a purposeful part of the product.
5. **Score:** use it to prioritize findings within this audit, not to compare unrelated products as if taste were universal.

A finding can lead to four legitimate outcomes:

- **Fix it:** restore product-specific hierarchy, interaction, content shape, or role-specific treatment.
- **Approve it as House Style:** use this for a reusable design choice that is intentional across the product.
- **Suppress it narrowly:** use a path/owner-specific suppression with a rationale for an exceptional case.
- **Leave it advisory:** uncertainty is preferable to encoding an unreviewed judgment as policy.

When asking an agent to refactor a finding, give it both generated artifacts. The brief now includes exact source locations, evidence snippets, and prominent coverage/applicability warnings; `report.json` remains the canonical machine input. Require the agent to verify the resolved source and CSS cascade before editing and to preserve behavior, accessibility, responsive behavior, focus order, and user workflows. The detector identifies convergence evidence; it does not prove a safe replacement design.

## A practical review loop

```sh
# Human-readable pass
ai-ui-slop scan . --format terminal --progress auto

# Inspect canonical evidence
jq '.scopes[] | {
  id,
  coverage,
  score: .repositoryProfile,
  findings: [.findings[] | {
    rule: .rule_id,
    owner,
    path,
    score,
    evidence
  }]
}' .ai-ui-slop/reports/report.json

# Explain one rule contract
ai-ui-slop explain framework-default-convergence

# Re-scan after fixes or reviewed policy changes
ai-ui-slop scan . --format terminal --progress auto
```

Commit the configuration and reviewed policy changes with the application. Do not commit `.ai-ui-slop/reports/` unless your workflow intentionally versions generated audit artifacts.

## Baselines and CI

Baselines are for preventing reviewed regressions, not for hiding the first audit. Clean up or explicitly review the advisory findings before creating one:

```sh
ai-ui-slop baseline create .
ai-ui-slop baseline accept . \
  --approver "maintainer-name" \
  --rationale "Reviewed current design debt and accepted it as the comparison point"
ai-ui-slop baseline check . --format json
```

Then change `"mode": "advisory"` to `"mode": "enforcement"` in the reviewed configuration. Enforcement requires a compatible Reviewed Baseline; a rule-pack change requires semantic migration review rather than silently accepting new fingerprints.

For pull requests, keep the policy and baseline in the protected base checkout:

```sh
ai-ui-slop scan ./pull-request \
  --trusted-policy-root ./protected-base \
  --jobs 4 \
  --max-wall-time-seconds 600 \
  --format json \
  --progress always
```

The root [composite Action](action.yml) accepts a separately installed, integrity-verified binary, uploads both report artifacts, writes a job summary, and preserves scanner exit codes.

### Exit codes

| Code | Meaning |
| ---: | --- |
| `0` | Scan or lifecycle command completed without an enforceable regression. Findings may still exist in advisory mode. |
| `1` | A compatible enforcement baseline has new or worsened enforceable findings. |
| `2` | Invalid command, configuration, lifecycle, path, or incompatible baseline. |
| `3` | Analysis or artifact coverage did not satisfy the active floor. |
| `4` | Local operational failure, such as installing the interrupt handler or writing an artifact. |
| `130` | Cancelled. Canonical artifacts are not committed for a cancelled scan. |

## Supported analysis

The scanner:

- analyzes configured `.js`, `.jsx`, `.ts`, and `.tsx` React sources with Oxc;
- recognizes Next App/Pages Router, static React Router, configured routes, and conventional React root-SPA boundaries;
- resolves bounded static Tailwind, CVA, inline-style, plain-CSS, import-graph, wrapper, and reachable-state evidence without executing the target repository;
- evaluates eleven alpha rules and fourteen built-in Page Archetypes;
- composes page evidence only through uniquely resolved local component edges under explicit resource ceilings;
- supports independent monorepository scopes, House Style, suppressions, rule dispositions, custom archetypes, Trusted Policy, and reviewed baselines; and
- reports Finding, Component, and scope-bounded Repository AI Slop Scores with named contributions.

See the [support matrix](docs/support-matrix.md) for exact boundaries. Inputs outside it are unsupported, not silently proven clean.

## Command reference

```sh
ai-ui-slop init ./repo
ai-ui-slop config validate ./repo --effective default
ai-ui-slop scan ./repo --format terminal --progress auto
ai-ui-slop scan ./repo --format json --progress never
ai-ui-slop explain effect-stacking
ai-ui-slop baseline create ./repo
ai-ui-slop baseline accept ./repo --approver maintainer --rationale "Reviewed debt"
ai-ui-slop baseline check ./repo --format json
ai-ui-slop feedback bundle ./repo
ai-ui-slop schema report
ai-ui-slop schema config
ai-ui-slop update check
ai-ui-slop version
```

## Verify this repository

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
python3 scripts/audit-requirements.py
scripts/audit-qualification.py --format json
cargo build --locked
scripts/fuzz-smoke.py --iterations 128
```

The public-seam tests cover discovery, graph and route adapters, static style resolution, rule boundaries, coverage and resource ceilings, hostile inputs, baselines, deterministic parallel execution, CLI lifecycles, progress, and report artifacts.

Full V1 qualification is deliberately separate from this local gate. The frozen sample sizes, human/agent trial rubrics, reference-runner contract, mutation command, evidence order, and fail-closed hosted validators are in the [qualification program](qualification/program.md). The current `0.14.0` candidate passes the local TEST-004 mutation gate at 80.5497%; Qualification Shot 3 wired reference, progress, and five-target native decisions without promoting local rehearsal data. A release may claim Full V1 only when `scripts/audit-qualification.py --require-complete` passes against committed evidence; the command is expected to fail on the current Implementation Candidate.

### Build-disk maintenance

Cargo writes reproducible build output to `target/`, which is ignored by Git. This project has many integration-test binaries linked against Oxc, so repeated builds under different source and compiler states can leave several generations of large artifacts there. The development and test profiles retain line-table diagnostics but disable full debug symbols and incremental caches to keep routine builds bounded.

Check and reclaim the generated space with:

```sh
du -sh target 2>/dev/null || true
cargo clean
```

`cargo clean` removes only Cargo-generated artifacts; the next build recreates what it needs. It does not remove source, configuration, reports outside `target/`, or Git history.

The latest real-repository calibration reduced EventCardSite from 31 mixed findings to six coherent Control Surface Homogenization findings while removing false framework, dialog-decoration, and stock-template clusters. See [EventCardSite calibration evidence](docs/evidence/EVENTCARD-CALIBRATION.md).

Design rationale and counterexample provenance from *Refactoring UI* are documented in the [reference traceability matrix](docs/references/refactoring-ui-traceability.md). The book informs explanations and counterexamples; it is not treated as a machine-enforceable taste specification.

## Reusable agent skill

The portable [Audit and Fix UI Slop skill](skills/audit-and-fix-ui-slop/README.md) can be copied into another repository's `.agents/skills/` directory or installed in `~/.codex/skills/`. It includes agent instructions, a conservative remediation guide, and a tested Bash runner that initializes, validates, scans, preserves exit codes, and records durable evidence without editing application source or accepting a baseline.

Invoke it from a compatible agent with:

```text
Use $audit-and-fix-ui-slop to audit this repository, fix only justified findings,
run the repository's checks, rescan, and report the before/after evidence.
```

Full V1 validation still requires executed reference-runner results, hosted release smoke tests, authenticated release assets, and blind customer calibration. The local mutation gate passes, but the other committed protocols remain unexecuted and are not evidence. See [qualification](qualification/README.md), [implementation-exit evidence](docs/evidence/V1-IMPLEMENTATION-EXIT-2026-08-01.md), [Shot 7](docs/shot-7.md), and its [local evidence](docs/evidence/SHOT7-LOCAL.md).

## License

Original project code and documentation are available under either the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE), at your option.
