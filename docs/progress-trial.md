# CLI Progress Display Trial

Status: candidate implemented; local automated performance gate passed; user evaluation and pinned-runner reproduction pending.

This trial evaluates whether the Discovery Prototype answers both “how far along is the scan?” and “what is the scanner doing?” without contaminating report output or adding excessive noise.

## Candidate A: monotonic plain bar

The candidate writes one monotonic overall bar plus one current-phase line to stderr. Each line contains:

- overall work completed;
- current phase;
- phase-local completed and total work when known;
- elapsed time;
- unresolved coverage count; and
- a concrete description of analyzer work.

It uses no ANSI cursor control in the current prototype, so redirected, CI, narrow-terminal, and captured output remain readable. Requested JSON stdout contains no progress bytes.

## Work-unit weights (`progress-weights/0.1.0`)

| Analyzer work | Overall units |
| --- | ---: |
| Repository discovery | 10 |
| Oxc parsing and static style/owner resolution | 60 |
| Route/archetype classification | 5 |
| Rule evaluation | 10 |
| Recurrence and score aggregation | 5 |
| Artifact validation and writing | 10 |
| **Total** | **100** |

Parsing and resolution are interleaved per file to bound retained parser memory. Each discovered file receives the same prototype weight. Route/archetype work is visibly marked not applicable rather than silently omitted.

These are trial weights, not measured V1 weights. They must be replaced or justified with benchmark evidence before the display is accepted.

## Automated evidence

The CLI seam test currently proves:

- the bar begins at zero and ends at 100%;
- overall progress never moves backward;
- phase-local counts reach the discovered file total;
- progress appears only on stderr;
- `--progress never` emits no stderr progress;
- progress-on and progress-off scans produce byte-identical JSON stdout; and
- JSON and Markdown artifacts remain projections of the same report.

## Human trial matrix

Run the same candidate against each workload and record observations without changing the display between trials.

| Workload | Required observation | Status |
| --- | --- | --- |
| Small scan | Does startup avoid noisy flicker and still explain the work? | Pending |
| Large scan | Does the bar advance credibly and are updates bounded? | Pending |
| Partial scan | Are coverage problems visible at the moment they arise and at completion? | Pending |
| Failing scan | Is the final successful phase and failure reason unambiguous? | Pending |
| Redirected/CI scan | Is plain output readable without cursor control? | Automated smoke evidence only |
| Narrow terminal | Do phase and detail fields degrade legibly? | Pending |
| Interrupted scan | Is terminal restoration and cancellation behavior correct? | Not implemented |

For each completed trial, the Design Authority should answer:

1. Could you tell how far along the scan was?
2. Could you tell what the analyzer was actually doing?
3. Was any line redundant, misleading, or too volatile?
4. Were coverage problems noticeable without overwhelming ordinary progress?
5. Would you keep this display enabled by default?

## Performance protocol

Acceptance requires at least 20 alternating progress-on/progress-off cold pairs on the pinned reference runner. Publish every paired delta, the median paired overhead, and a 95% confidence interval. Median overhead must not exceed 2%. Report bytes, Finding order, scores, and exit status must match within every pair.

A local 20-pair run passed the median overhead gate; see `docs/evidence/PROGRESS-010.md`. Pinned reference-runner reproduction remains required before release.
