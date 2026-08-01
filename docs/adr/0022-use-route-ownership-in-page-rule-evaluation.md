# ADR 0022: Use route ownership in page-rule evaluation

Status: Accepted

Date: 2026-07-31

## Decision

Rule pack `1.0.0-beta.7` passes exact discovered route path/owner pairs into Template Convergence. A framework route may therefore use an owner such as `Home` without relying on a `Page`, `Screen`, or `View` suffix. Other components in the same page module are not promoted.

Next.js adapters recognize both root-level and `src/` App and Pages Router layouts, including named function declarations and named identifiers used as default exports. Files in test/e2e directories and `*.test.*`, `*.spec.*`, or `*.stories.*` modules remain eligible source when otherwise configured, but they never become application routes. Bento evidence from explicit spans is limited to content/container elements and excludes action controls.

## Context

The nine-repository completion audit exposed three independent false boundaries. A valid `src/app/page.tsx#Home` route was labeled as a generic filesystem route and was ineligible for page rules. Internal component tests with names such as `CatalogView.test.tsx` were emitted as application routes with fabricated filename-derived owners. A grid-spanning action button inflated a Template Convergence signature with bento evidence.

## Consequences

- Route discovery becomes an explicit input to rule evaluation instead of a report-only parallel result.
- Route classification now runs before source rule evaluation, without executing the target repository.
- Existing baselines require semantic migration review because route eligibility and Template Convergence evidence can change.
- Test sources still contribute honest parse/style/graph coverage; only navigation classification excludes them.
