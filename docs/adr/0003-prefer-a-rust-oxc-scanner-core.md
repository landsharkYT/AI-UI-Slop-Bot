# Prefer a Rust and Oxc scanner core

Status: Proposed

AI UI Slop Bot will prefer a single Rust-native core and CLI using Oxc, with GitHub Actions acting only as a thin launcher. This accepts higher initial implementation and release-engineering cost in exchange for faster repository-wide parsing and traversal, lower memory use, a native standalone executable, and avoiding a long-term TypeScript/Rust split; the decision becomes accepted only when the requirements-defined feasibility benchmark confirms AST fidelity, hard cross-file cases, 500,000-line parsing and graph construction within 15 seconds and 750 MB, deterministic cross-platform packaging, a compressed binary no larger than 30 MiB, and no project-authored unsafe Rust.
