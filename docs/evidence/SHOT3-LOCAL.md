# Shot 3 Local Qualification Evidence

Status: local automated gates passed; hosted, mutation/fuzz, and human validation remain pending.

Date: 2026-07-29

Scanner `0.3.0` was built with Rust `1.96.0` on Linux x86-64. The release binary SHA-256 was `d566db2cef876ac48bb8ea6bddffa84f4eed4018fdf316402267b913cd78b8e7`, its uncompressed size was 3,626,984 bytes, and `Cargo.lock` SHA-256 was `0597bfd4ba2ba4a8789c793c04ed9da5dd5aa007a3588cc5c45e2d9020a1be43`.

The full local gate included formatting, Clippy with warnings denied, 43 seam tests, the 390-ID requirements audit, release compilation, JSON/YAML validation, and SPDX generation.

The committed 500-file, 500,500-line synthetic workload produced:

```json
{"fixtureVersion":"1","fileCount":500,"lineCount":500500,"elapsedMs":85,"peakRssKiB":14788,"binary":"target/release/ai-ui-slop"}
```

The twenty-pair local progress trial produced byte-identical reports, median paired overhead of -0.1018%, an empirical interval of -4.8715% to 5.7288%, and canonical report SHA-256 `c5786a098ec5639c77dfb07e3270f9668c5f7d1d2dee5425eba73f26f1253de6`. The negative median is ordinary measurement noise, not a claim that rendering improves performance.

These measurements establish local feasibility only. They do not satisfy the pinned reference-runner, second 2,000-file workload, five-platform, mutation/fuzz, customer-calibration, or agent-cleanup release gates.
