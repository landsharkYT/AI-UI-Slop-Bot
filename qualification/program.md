# Full V1 qualification program

This program turns the frozen contracts in `protocol.json` and `reference-runner.json` into an evidence sequence. Passing local tests or creating a workflow does not satisfy a hosted or external gate.

## Execution order

1. Run the ordinary local qualification suite on the candidate revision.
2. Run the mutation gate once without `--iterate`; normalize it with `scripts/mutation-score.py`.
3. Dispatch **Manual Full V1 qualification / reference-benchmark**. Retain `benchmark.json` and the passing `reference-decision.json` from the same artifact.
4. Dispatch **Manual Full V1 qualification / progress** on the same candidate revision and resolved runner image. Retain `progress.json` and the passing `progress-decision.json`.
5. Dispatch **Release native binaries** on the candidate revision. Manual dispatch builds and smokes all five targets without publishing. Retain the five target records and passing `native-qualification.json`.
6. Freeze calibration case IDs, repositories, design families, holdout membership, trial pairs, and reviewer assignments before collecting labels.
7. Execute the maintainer and agent trials exactly as specified in `maintainer-trial.md` and `agent-trial.md`; seal raw redacted records before scanner scoring or reviewer unblinding.
8. Update only the verification rows directly supported by durable evidence, run `scripts/audit-qualification.py`, and independently review the release decision.
9. Create an immutable signed tag only after every prerequisite gate passes. A tag run repeats the five-target native qualification before the publish job can create release assets, digests, SBOM, and attestations.

## Automated evidence commands

The hosted workflows invoke these validators. A nonzero result is a failed gate, not a request to edit the evidence.

```sh
scripts/qualification-program.py reference benchmark.json \
  --output reference-decision.json
scripts/qualification-program.py progress progress.json \
  --output progress-decision.json
scripts/qualification-program.py native native-record-directory \
  --output native-qualification.json
```

Reference and progress evidence must identify the frozen four-logical-processor GitHub runner and a concrete `ImageVersion`; `local-unqualified` is rejected. The reference decision independently checks required fields, workload sizes, elapsed time, memory, and scanner exits. The progress decision independently checks runner identity, exactly 20 alternating pairs, report and outcome equivalence, the recomputed median, and the interval shape. The native decision requires one qualified, deterministic smoke record for every frozen target and a common revision, scanner version, and rule-pack version.

`scripts/native-smoke.py BINARY TARGET OUTPUT` records the binary digest and size, version contract, runner identity, and two fresh-fixture scan digests. It never substitutes cross-compilation for execution on the target runner.

## Evidence promotion

- Local rehearsal artifacts remain under `target/qualification/` and cannot promote hosted rows.
- Hosted artifacts must be attached to an immutable candidate revision and copied into a reviewed durable evidence record before a matrix status changes.
- A precomputed `passes...` boolean is never sufficient; qualification validators recompute decisions from raw fields.
- Missing targets, partial runs, local runner identities, malformed data, mixed revisions, changed report bytes, timeouts where disallowed, and failed commands fail closed.
- The final `--require-complete` audit is necessary but not sufficient: it checks ledger completeness, while reviewers still verify that each evidence record supports its cited requirement.

## Human calibration boundary

The automated program cannot manufacture customer satisfaction. Rule precision/recall/useful-yield, Page Archetype calibration, maintainer usefulness, and agent cleanup preference remain external blind trials. Their sample sizes and pass thresholds are frozen in `protocol.json`; their collection procedures are the two trial documents in this directory.
