# Version the coverage correction as 0.15

Status: Accepted

The coverage and ownership correction ships as scanner `0.15.0` with canonical report schema `9` and a new beta rule-pack version. Existing Reviewed Baselines remain readable for a migration preview but are incompatible with enforcement until explicitly reviewed and regenerated. The preview reports added, removed, re-owned, and evidence-changed Findings together with coverage-status changes; no command silently rewrites or accepts a baseline. This treats newly visible evidence and corrected ownership as semantic changes rather than disguising them as implementation-only fixes.
