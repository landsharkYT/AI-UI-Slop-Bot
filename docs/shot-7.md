# Shot 7 — V1 Implementation Closeout

Status: superseded by the `0.13.0` blocker closeout; external release validation remains open.

Shot 7 closes the remaining implementation seams identified after Shot 6. It does not convert unrun customer studies or hosted-platform jobs into evidence.

## Implemented

- cooperative cancellation shared by the CLI, repository scheduler, and source scanner;
- first-interrupt exit `130` with no canonical report artifacts, plus immediate second-interrupt termination;
- AST-node accounting through the Oxc visitor and parser-arena accounting through the allocator;
- canonical per-scope resource usage in report schema 6;
- explicit incomplete-coverage diagnostics for AST, analysis-memory, directory-depth, wall-time, and diagnostic ceilings;
- deterministic per-reason diagnostic truncation;
- local wall-time control and Linux composite-Action outer memory control;
- final `0.7.0` CLI/version/config contract;
- schema 6 and Shot 7 public-seam regression coverage.

## Local qualification boundary

The Shot 7 local suite passed and established a breadth-complete build suitable for use-case testing. Subsequent audits identified four V1 Implementation Blockers: bounded static Tailwind theme/custom-utility semantics, supported variant reachability, symbol-aware re-export provenance, and component-level shared-primitive attribution. The build also does not satisfy customer-satisfaction thresholds, committed reference-runner performance gates, five-platform hosted smoke tests, authenticated tag-release verification, or the remaining broad fuzz/mutation program.

After the four implementation blockers are resolved and locally verified, the build may be called a **V1 Implementation Candidate**. The remaining external gates determine whether it may be called **Validated MVP** or **Full V1**.

The `0.13.0` follow-up resolved those blockers through public-seam regressions and restored **V1 Implementation Candidate** status. The external gates remain unchanged.

Advisory Use-Case Trials may run before those blockers close because real-repository feedback can improve their implementation and later calibration. These trials must remain non-enforcing, versioned, and explicit about known coverage limits; they are not release-acceptance evidence by themselves.
