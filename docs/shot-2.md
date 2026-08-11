# Shot 2: Technical Hardening Candidate

Status: implemented locally; external validation and release qualification remain pending.

Shot 2 deepens the existing scanner, repository-analysis, and CLI seams rather than adding a second analysis engine.

## Implemented

- bounded reachable-state families for literal, static-template, conditional, nested-conditional, and configured `clsx`/`classnames`/`cn`/`twMerge`-shaped composition;
- reachable-state identity in Findings and fingerprint algorithm 2;
- a deterministic typed repository graph for modules, rendered components, routes, Page Archetypes, and approved primitives;
- measured component-graph coverage with unresolved-edge diagnostics and an edge ceiling;
- per-file, per-scope source, graph, diagnostic, JSON, Markdown, and scope-count budgets;
- expired and unmatched Suppression diagnostics plus unmatched approved-primitive diagnostics;
- semantic previews for incompatible baseline migrations;
- protected Trusted Policy roots that keep checkout policy changes from weakening their own analysis;
- bounded parallel Analysis Scope scheduling with byte-equivalent canonical output across job counts;
- report schema 2, baseline schema 2, rule pack `1.0.0-beta.1`, and scanner `0.2.0`;
- five-target native release workflow, SHA-pinned Actions, authenticated GitHub attestations, digest manifests, and SPDX generation; and
- reproducible 500,000-line benchmark, requirements-audit, and customer-calibration evidence formats.

## Not release evidence

Implementation cannot manufacture the evidence that requires external people, hardware, or successful hosted workflows. The following remain release gates:

- blind rule and per-archetype precision/recall/yield calibration;
- seven external-maintainer satisfaction trials and ten fresh agent cleanup trials;
- twenty-pair progress overhead measurement and Design Authority progress review;
- the Rust/Oxc versus TypeScript feasibility comparison;
- actual five-platform workflow runs, artifact smoke tests, attestation verification, and minimum-OS qualification;
- cancellation/signal qualification and allocator-accounted live-memory enforcement;
- complete CVA, Tailwind configuration/CSS import, alias/export, and framework-route adapter fixtures; and
- mutation, fuzz, hostile-presentation, and broad corpus results required by `requirements.md`.

Until those records exist, this repository is a Shot 2 technical candidate, not Validated MVP or Full V1.
