# TECH-001 Local Feasibility Evidence

Status: partial evidence; release gate remains pending.

Date: 2026-07-29

The committed `scripts/benchmark.sh` generated 500 files and 500,500 TypeScript/JSX source lines, then measured a clean optimized scan with progress disabled.

Local runner:

- Linux x86-64, kernel `6.12.96-1-MANJARO`;
- 13th Gen Intel Core i7-13700HX, 24 logical CPUs;
- Rust `1.96.0` (`ac68faa20`, LLVM 22.1.2);
- scanner `0.2.0`;
- benchmark script SHA-256 `52941e25352a259f5b9863541701ea8ff44a042f3d0aa2336cd7fac7ad60f4d6`;
- Cargo.lock SHA-256 `e524df25aa509037673778246d350cb3252465b2b69a0979962627f28efe0cba`; and
- release binary SHA-256 `8b3039b44d3ac0ba59c80f59ea41cdcfb223190c8bad47106126f2505c483c65`.

Observed result:

```json
{"fixtureVersion":"1","fileCount":500,"lineCount":500500,"elapsedMs":77,"peakRssKiB":14284,"binary":"target/release/ai-ui-slop"}
```

The uncompressed binary was 3,430,952 bytes. This local run satisfies the numerical 15-second, 750-MiB, and 30-MiB feasibility thresholds for this synthetic workload.

It does not accept ADR 0003 by itself. The workload deliberately isolates parsing and ordinary graph construction; hard CVA/import/route fixtures, the TypeScript comparison, pinned reference-runner reproduction, five-platform builds, archive sizes, and hosted Action smoke tests remain required.
