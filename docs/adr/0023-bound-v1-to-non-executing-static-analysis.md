# Bound V1 to non-executing static analysis

Status: Accepted

Full V1 guarantees the documented, bounded static-analysis surface rather than attempting general frontend execution or complete framework interpretation. Runtime CVA selections, additional non-executing recognition of arbitrary render-transforming HOCs or Tailwind plugin semantics, a general CSS cascade, CSS Modules, and CSS-in-JS analysis are post-V1 candidates because supporting them would expand V1 into an open-ended browser and framework analysis project. Executing target code or configuration remains a product safety boundary. Unsupported forms must remain visible as coverage loss. Platform qualification, reference-runner performance, release trust, required fuzz and mutation evidence, and customer calibration remain mandatory release gates; passing local verification makes a build a V1 Implementation Candidate, not Full V1.

Bounded static Tailwind theme and custom-utility semantics, supported variant reachability, symbol-aware re-export provenance, and component-level shared-primitive attribution remain V1 features. They materially affect whether Findings are correct and actionable, so incomplete implementations are V1 Implementation Blockers rather than deferred enhancements.

Advisory Use-Case Trials may run while those blockers remain so real-repository evidence can guide the work. Such trials cannot enable baseline enforcement or support V1 qualification claims.

The `0.15.0` coverage-correction pass does not reopen this boundary. It adds accurate attribution for the existing plain-CSS surface, bounded conditional and compound-selector reasoning, relevant custom-property provenance, and structured Coverage Diagnostics; CSS Modules, CSS-in-JS execution, runtime theme providers, browser layout, and a general computed-style or cascade engine remain outside the pass.
