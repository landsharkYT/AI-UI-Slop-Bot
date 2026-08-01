# Qualification Shot 1 evidence

Status: qualification infrastructure implemented and locally verified; Full V1 remains unqualified.

Shot 1 establishes reproducible ways to gather evidence without substituting harness existence for the evidence itself:

- `scripts/audit-qualification.py` audits every verification-matrix status, evidence path, and requirement citation; `--require-complete` fails while any requirement is pending.
- `tests/qualification_property_seam.rs` exercises class-order invariance and malformed-source isolation through the public scanner surface.
- `scripts/fuzz-smoke.py` drives deterministic valid, malformed, and invalid-UTF-8 repositories through the real CLI twice and rejects internal errors or nondeterminism.
- `scripts/mutation-score.py` normalizes cargo-mutants outcomes, counts timeouts as uncaught viable mutants, and enforces the provisional 80% rule/policy score.
- `scripts/benchmark.sh` commits separate 2,000-file and 500,000-line workloads and records the runner, toolchain, scanner, rule-pack, fixture, elapsed-time, and peak-memory context.
- `qualification/protocol.json`, `qualification/reference-runner.json`, and the frozen maintainer/agent protocols fix the sample sizes, pass conditions, blinding, and failure treatment before outcomes are known.
- the local qualification workflow runs on pull requests and `main`; the manual workflow exposes the expensive mutation, reference-benchmark, and progress gates without pretending they ran.

Local verification on 2026-07-31 passed formatting, Clippy with warnings denied, all Cargo test targets, ten Python harness tests, the 395-ID requirements audit, and the evidence-ledger consistency audit. The 128-case deterministic fuzz smoke run completed with 64 normal and 64 explicit incomplete-analysis outcomes and no internal error or nondeterminism.

A local, debug-build rehearsal of the committed workloads processed 2,000 files in 184 ms at 15,520 KiB peak RSS and 500,000 lines in 994 ms at 14,648 KiB peak RSS. Its evidence correctly labels the runner `local-unqualified`; these observations do not satisfy `QUALITY-002`, `QUALITY-004`, `TEST-004`, `TEST-007`, `V1-AC-017`, `V1-AC-022`, hosted-platform gates, release-authentication gates, or any human/customer gate.

At this checkpoint the ledger reports 395 requirements: 3 `local-pass`, 392 `pending`, and zero invalid evidence rows. No pending verification-matrix status was promoted by Shot 1.
