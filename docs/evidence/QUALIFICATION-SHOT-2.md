# Qualification Shot 2 evidence

Status: automated local qualification strengthened; Full V1 remains unqualified.

Shot 2 exercised the evidence paths created in Shot 1 and closed the provisional mutation gate:

- a complete 949-mutant cargo-mutants 27.0.0 run scored 80.6311%, counting five timeouts against the score;
- 21 public-seam Rust tests now cover policy/baseline identity, migration and thresholds; style and route recognition; coverage floors; cancellation; calendar boundaries; score bands; Tailwind conflicts; and exact style-file byte ceilings;
- the mutation evidence normalizer resolves cargo-mutants' nested output, rejects incomplete runs, and refuses accumulated `--iterate` output as release evidence;
- a release-build local benchmark completed the 2,000-file workload in 55 ms at 14,692 KiB peak RSS and the 500,000-line workload in 122 ms at 14,444 KiB peak RSS;
- the 20-pair progress trial preserved identical report hashes and measured -0.0397% median paired overhead, passing the provisional 2% median gate;
- a deterministic 1,024-case fuzz smoke run produced 512 successful reports and 512 explicit incomplete-analysis results, with no internal-error or nondeterminism rejection.

The benchmark and progress results are local rehearsals on an Intel Core i7-13700HX Manjaro runner labeled `local-unqualified`. They do not substitute for the pinned reference runner. The mutation result is recorded separately in [TEST-004](TEST-004.md), which is promoted to local-pass; other qualification rows remain unchanged.

Full V1 still requires the committed reference-runner executions, hosted native-target and authenticated-release checks, and blind customer/Page Archetype and agent-cleanup calibration. Shot 2 proves the automated harness can produce honest evidence; it does not convert unexecuted protocols into evidence.
