# GitHub Action — Shot 2 Candidate

The root `action.yml` is a thin composite adapter over the same native scanner used locally. It does not build or download code and therefore requires an absolute path to a separately installed, integrity-verified binary.

For pull-request enforcement, check out the protected target revision into a second worktree and pass that directory as `trusted-policy-path`. Scope assignment, House Style, Suppressions, thresholds, resource ceilings, custom archetypes, and the Reviewed Baseline are then read from the protected revision. Checkout changes remain visible as proposals but cannot weaken their own scan.

Caller workflow policy:

```yaml
permissions:
  contents: read

steps:
  - uses: actions/checkout@<full-reviewed-commit-sha>
    with:
      path: checkout
      persist-credentials: false
  - uses: actions/checkout@<full-reviewed-commit-sha>
    with:
      ref: ${{ github.event.pull_request.base.sha }}
      path: trusted-base
      persist-credentials: false
  - name: Install and verify pinned ai-ui-slop release
    run: ./your-reviewed-install-step
  - uses: your-org/ai-ui-slop-bot@<immutable-commit-sha>
    with:
      binary-path: ${{ runner.temp }}/ai-ui-slop/ai-ui-slop
      trusted-policy-path: ${{ github.workspace }}/trusted-base
```

Use `pull_request`, not `pull_request_target`, for contributor-controlled source. The Action uploads canonical JSON and Markdown, appends the Refactoring Brief to the job summary, requests no write permission, and preserves scanner exit codes.

The release workflow defines five native build jobs, SHA-256 manifests, an SPDX SBOM, and GitHub attestations. Those are implementation assets, not qualification evidence until a real immutable tag workflow succeeds and every artifact is smoke-tested.
