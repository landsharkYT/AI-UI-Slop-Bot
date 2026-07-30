# Shot 4 Local Qualification Evidence

Status: local automated gates passed; hosted, mutation/fuzz, and human validation remain pending.

Date: 2026-07-29

Scanner `0.4.0` was built with Rust `1.96.0` on Linux x86-64. The release binary SHA-256 was `604009dd24997d226bd32f619a679775ba08e01f972f677cf1cc5115abddf3ea`, its uncompressed size was 3,667,400 bytes, and `Cargo.lock` SHA-256 was `03ed3f13efb4380d3870c7f969b83a8f83f98c7fbc5859222b73a98fce4180b1`.

The full local gate included formatting, Clippy with warnings denied, 47 seam tests, the 390-ID requirements audit, release compilation, JSON schema validation, and SPDX generation.

The committed 500-file, 500,500-line synthetic workload produced:

```json
{"fixtureVersion":"1","fileCount":500,"lineCount":500500,"elapsedMs":100,"peakRssKiB":14648,"binary":"target/release/ai-ui-slop"}
```

The twenty-pair local progress trial produced byte-identical reports, median paired overhead of 1.1582%, an empirical interval of -7.4967% to 8.4333%, and canonical report SHA-256 `bc7756b87f6fc8791f1cccf2f98ac2249f76aefb2c26e5f23c3ab58740750c19`.

These measurements establish local feasibility only. They do not satisfy the pinned reference-runner, second 2,000-file workload, five-platform, mutation/fuzz, customer-calibration, or agent-cleanup release gates.
