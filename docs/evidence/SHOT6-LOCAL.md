# Shot 6 Local Qualification Evidence

Status: local automated gates passed; hosted, mutation/fuzz, and human validation remain pending.

Date: 2026-07-29

Scanner `0.6.0` was built with Rust `1.96.0` on Linux x86-64. The release binary SHA-256 was `1cc0d4e22e1b39295e14deebd4068ec647243c6269bbc6045720c20aaf7965f6`, its uncompressed size was 3,960,016 bytes, and `Cargo.lock` SHA-256 was `817b8c652fb45621b40b7ae8e8d9460586f0b025c62bcfdcb1efc44a8b663ee1`.

The full local gate included formatting, Clippy with warnings denied, 60 seam tests, the 390-ID requirements audit, release compilation, report/config schema validation through the CLI suite, and deterministic one-worker/eight-worker Shot 6 comparison.

The fixed 500-file, 500,500-line synthetic workload produced:

```json
{"fixtureVersion":"1","fileCount":500,"lineCount":500500,"elapsedMs":100,"peakRssKiB":14644,"binary":"target/release/ai-ui-slop"}
```

The twenty-pair local progress trial produced byte-identical reports, median paired overhead of 0.5088%, an empirical interval of -8.6335% to 15.3217%, and canonical report SHA-256 `4012789353f369752ae127be1de5789302282266c5a6f5dbeece735a42069f8b`.

These measurements establish local feasibility only. They do not satisfy the pinned reference-runner, second 2,000-file workload, five-platform, mutation/fuzz, customer-calibration, or agent-cleanup release gates.
