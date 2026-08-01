# Full V1 qualification

This directory freezes the release-decision rules before external outcomes are known. It does not contain fabricated customer evidence and does not make the current Implementation Candidate a Full V1 release.

The implementation and local rehearsal records for this harness are [Qualification Shot 1](../docs/evidence/QUALIFICATION-SHOT-1.md), [Qualification Shot 2](../docs/evidence/QUALIFICATION-SHOT-2.md), and [Qualification Shot 3](../docs/evidence/QUALIFICATION-SHOT-3.md). The completed local mutation gate is recorded under [TEST-004](../docs/evidence/TEST-004.md). The end-to-end execution and promotion sequence is the [Full V1 qualification program](program.md).

`protocol.json` is the machine-readable gate contract. `reference-runner.json` fixes the runner class, CPU allocation policy, and evidence fields. The two trial protocols fix ordering, blinding, forms, and failure treatment.

## Evidence order

1. Commit case identifiers, repository revisions, design-family assignments, holdout membership, agent-trial pairs, and reviewer assignments.
2. Collect independent labels without scanner scores, bands, other reviewers' judgments, or cleanup identity.
3. Seal the labels and record their digest.
4. Run the pinned scanner and rule pack without tuning against the sealed holdout.
5. Calculate every rule and archetype result separately. Preserve rejected, ambiguous, abstained, timed-out, and failed records.
6. Commit raw redacted records and derived results. A failed gate remains failed for that release-decision cycle.

The qualification ledger is audited with `scripts/audit-qualification.py`. `--require-complete` is the final release gate and is expected to fail until every row in `docs/requirements-verification.md` has evidence.

## Automated entrypoints

```sh
cargo build --locked
scripts/fuzz-smoke.py --iterations 128
cargo mutants --no-shuffle --output target/qualification/mutants \
  --file src/app.rs --file src/baseline.rs --file src/policy.rs --file src/style.rs
scripts/mutation-score.py target/qualification/mutants \
  --output target/qualification/mutation-score.json
scripts/qualification-program.py reference \
  target/qualification/benchmark/benchmark.json \
  --output target/qualification/reference-decision.json
scripts/qualification-program.py progress \
  target/qualification/progress.json \
  --output target/qualification/progress-decision.json
scripts/qualification-program.py native target/qualification/native \
  --output target/qualification/native-decision.json
```

Timeouts count as viable mutants that were not caught. Unviable mutants are reported but excluded from the score. Surviving mutants require substantive review; suppressing them merely to reach 80% is not qualification. The scorer also rejects incomplete runs and output accumulated with `cargo mutants --iterate`; qualification evidence must come from one complete run.
