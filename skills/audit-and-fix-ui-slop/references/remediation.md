# Evidence and remediation guide

## Reading order

1. Confirm the scope is applicable.
2. Inspect coverage and unresolved diagnostics.
3. Inspect each finding's rule, owner, path, line, reachable state, signature, evidence snippets, score contributions, and confidence.
4. Use the Refactoring Brief to plan work only after confirming it agrees with canonical JSON.
5. Treat the repository score as prioritization within one audit, not a cross-product quality grade.

Optional `jq` triage:

```sh
jq '.scopes[] | {
  id,
  coverage,
  score: .repositoryProfile,
  unresolved: .styleAdapter.unresolved,
  findings: [.findings[] | {
    rule: .rule_id,
    owner,
    path,
    line,
    score,
    confidence,
    signature,
    evidence
  }]
}' .ai-ui-slop/reports/report.json
```

## Dispositions

### Fix

Use when exact evidence shows the same decorative or structural recipe being applied without regard for content, role, hierarchy, or product identity. Change the smallest coherent design seam. Preserve behavior and accessibility, then verify visually when the repository has a supported visual test workflow.

### House Style

Use for an intentional reusable design-system choice. Require Design Authority review. Record the intent and the exact approved signal, value, or primitive; do not approve a vague category merely to silence findings.

### Narrow suppression

Use for a deliberate exception tied to a specific path or owner, with rationale and expiry where appropriate. Never use a repository-wide suppression as a shortcut.

### Advisory or unresolved

Use when the evidence is ambiguous, the scanner could not resolve relevant styling or ownership, or the product decision lacks an authorized reviewer. Preserve the diagnostic and explain what additional evidence would resolve it.

## Fix principles

- Restore hierarchy before adding decoration.
- Let content and interaction determine container shape.
- Remove redundant shells before inventing new visual effects.
- Differentiate controls and surfaces by role, not random variation.
- Prefer product-specific information architecture, terminology, and workflows over generic dashboard or landing-page formulas.
- Keep repeated treatments when repetition communicates a real system.
- Do not apply isolated advice from *Refactoring UI* as a machine-enforced rule. Use it as design rationale and counterexample context.

## Verification

After repository checks pass, rescan and compare:

- exact finding identities and evidence;
- applicability and coverage;
- new or worsened findings;
- resolved versus newly unresolved diagnostics;
- behavior, accessibility, responsive layout, focus order, and workflows; and
- score contributions only as a secondary summary.

A lower score with worse coverage is not an improvement. A finding removed by hiding source, breaking ownership resolution, or adding a suppression is not a code fix.
