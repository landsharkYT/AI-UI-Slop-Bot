# Calibration Evidence

This directory is the durable input format for Shot 2 customer calibration. It is intentionally separate from executable scanner policy: reviewer labels may justify later rule-pack changes, but they are never loaded at scan time.

Each case directory must contain:

- `manifest.json`, validated against `manifest.schema.json`;
- repository source or an immutable source revision;
- one or more rendered references for human evaluation;
- expected and rejected Finding labels with rationales;
- Page Archetype and design-family labels;
- reviewer judgments, including disagreements and abstentions; and
- the scanner report produced before labels are revealed.

Holdout assignment must be declared before threshold calibration. A repository or closely related design family cannot appear in both calibration and holdout sets.

No real customer evidence is committed by Shot 2 implementation itself. The Design Authority must recruit reviewers and record the required blind trials before V1 release claims are allowed.
