# ADR 0009: Expand V1 calibration with two convergence rules

Status: Accepted

Date: 2026-07-30

## Decision

Add Framework Default Convergence and Control Surface Homogenization to rule pack `1.0.0-beta.5`, expanding the V1 candidate from nine to eleven executable rules.

Framework Default Convergence requires a multi-signal stock framework recipe across at least three component owners. Control Surface Homogenization requires compact surface traits across at least eight elements and three structural roles. Both honor approved House Style signals and use semantic evidence rather than class names.

Responsive behavior, accessibility, and general usability remain non-scoring concerns. They may become advisories later, but they are not evidence of aesthetic convergence by themselves.

## Context

The original nine rules correctly identified heavily decorated and stock page compositions, but real-repository calibration exposed two blind spots. A restrained Tailwind application could repeat a slate/sky/rounded/popover recipe without triggering decoration-heavy rules. A dense workbench could flatten its header, navigation, controls, content, inspector, and status regions into one compact bordered treatment without using fashionable effects.

These are aesthetic-convergence patterns rather than generic quality defects. The rule boundaries deliberately protect isolated framework use and coherent control families.

## Consequences

- ReactPDFRedactor and the OSM utility are no longer clean negative controls.
- The rule catalog, requirements, support matrix, version output, fixtures, and baseline compatibility advance together.
- Existing baselines require semantic migration review because the rule-pack version changed.
- Customer calibration remains required; these rules are alpha contracts, not universal design judgments.
