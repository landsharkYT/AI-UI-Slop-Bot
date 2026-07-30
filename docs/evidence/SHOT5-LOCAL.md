# Shot 5 Local Qualification Evidence

Status: local automated gates passed; hosted, mutation/fuzz, and human validation remain pending.

Date: 2026-07-29

Scanner `0.5.0` was built with Rust `1.96.0` on Linux x86-64. The release binary SHA-256 was `2b20c5c6d73afd23cf6148cddbcb4712ef846434766580ce4dca0c2e61314bbe`, its uncompressed size was 3,859,400 bytes, and `Cargo.lock` SHA-256 was `05f188807efbd49a6e61104c383d3d1bd9a38712141d9cdf1159981bcc5de3a6`.

The full local gate included formatting, Clippy with warnings denied, 55 seam tests, the 390-ID requirements audit, release compilation, JSON schema validation, and SPDX generation.

The committed 500-file, 500,500-line synthetic workload produced:

```json
{"fixtureVersion":"1","fileCount":500,"lineCount":500500,"elapsedMs":103,"peakRssKiB":14456,"binary":"target/release/ai-ui-slop"}
```

The twenty-pair local progress trial produced byte-identical reports, median paired overhead of 0.6434%, an empirical interval of -24.5979% to 8.4261%, and canonical report SHA-256 `57f3f7794563bab7ddc945939945b8e17b6cf85d646375cb8265006c88801e21`.

These measurements establish local feasibility only. They do not satisfy the pinned reference-runner, second 2,000-file workload, five-platform, mutation/fuzz, customer-calibration, or agent-cleanup release gates.
