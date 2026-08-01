# V1 implementation-exit checkout

Status: implementation exit passed locally on 2026-08-01; Full V1 release qualification remains pending.

## Frozen candidate

- Candidate revision: `b034b33bcad3cd116a029f3df012f7e82cf9cb03`
- Scanner: `0.14.0`
- Report schema: `8`
- Rule pack: `1.0.0-beta.8`
- Cardification contract: `0.2.0-alpha`
- Upgrade path: [0.13 to 0.14 migration](../migrations/0.13-to-0.14.md)

The candidate includes the completed bounded V1 implementation and one qualification-only regression test added after the first mutation pass honestly missed the gate by 0.0846 percentage points. No production source changed during mutation remediation.

## Local verification

The checkout passed:

- `cargo fmt --all -- --check`;
- `cargo clippy --locked --all-targets --all-features -- -D warnings`;
- `cargo test --locked --all-targets --all-features`;
- `python3 -m unittest discover -s scripts/tests -p 'test_*.py'`;
- `python3 scripts/audit-requirements.py`;
- `scripts/audit-qualification.py --format json`;
- `scripts/fuzz-smoke.py --iterations 128`;
- `cargo build --locked`; and
- `cargo build --locked --release`.

The requirements audit found 395 exact IDs with valid schemas and ADRs. The qualification audit retained four `local-pass` rows and 391 `pending` rows with zero invalid evidence rows. Pending rows were not promoted from implementation tests or local rehearsals.

## Mutation qualification

The complete non-iterative TEST-004 rerun used cargo-mutants 27.0.0 and tested all 976 generated mutations: 762 caught, 179 missed, 5 timed out, and 30 unviable. Timeouts remain in the viable denominator. The resulting 762 / 946 score is **80.5497%**, passing the frozen 80% floor.

The normalized result is [committed here](artifacts/mutation-score-v014-exit.json). The raw ignored `outcomes.json` SHA-256 is `f4dfe3d3648cba3192e075f63c43d252dcf4f5d309fc2e37a09f8c3bbbad72e3`; the normalized artifact SHA-256 is `93d7407337c0303375af89653353afd45bd7037c6eac0670477cf0df35e64dfb`. Full commands and the failed first-pass disposition are recorded in [TEST-004](TEST-004.md).

## Real-repository readiness signal

The latest adjacent-repository pass remains useful for use-case testing rather than universal taste validation. ReactPDFRedactor retains a small number of review candidates after false sidebar/cascade evidence was removed; OSM–GeoJSON to MarkdownMap reports no finding but remains coverage-limited; EvacLogix retains ten repeated-shell positives; and scopes without supported React input report `not_applicable` instead of a false clean result. Detailed historical counts and limitations remain in the [completion audit](COMPLETION-AUDIT-2026-07-31.md) and [real-repository hardening evidence](REAL-REPOSITORY-HARDENING.md).

## Exit decision

The repository is ready to leave implementation and begin structured advisory use-case testing. This is not a Full V1 release claim. Enforcement should remain disabled until the applicable external gates pass.

No V1 release tag is created by this checkout. Full V1 still requires durable passing evidence for the committed reference-runner benchmark, hosted native and authenticated release checks, blind maintainer/customer calibration, Page Archetype calibration, and agent-cleanup trials. `scripts/audit-qualification.py --require-complete` is therefore expected to fail on this candidate.
