# GitHub Action — Shot 1 Alpha

The root `action.yml` is a thin composite adapter over the same native scanner used locally. It does not build or download code and therefore requires an absolute path to a separately installed, integrity-verified binary.

Caller workflow policy:

```yaml
permissions:
  contents: read

steps:
  - uses: actions/checkout@<full-reviewed-commit-sha>
  - name: Install and verify pinned ai-ui-slop release
    run: ./your-reviewed-install-step
  - uses: your-org/ai-ui-slop-bot@<immutable-commit-sha>
    with:
      binary-path: ${{ runner.temp }}/ai-ui-slop/ai-ui-slop
```

Use `pull_request`, not `pull_request_target`, for contributor-controlled source. The Action uploads canonical JSON and Markdown, appends the Refactoring Brief to the job summary, requests no write permission, and preserves scanner exit codes.

Authenticated release download, manifest verification, SBOM/provenance production, and exact multi-platform release workflows remain Shot 2 work. Until those exist, every caller owns the installation and trust bootstrap and must not describe the alpha as a turnkey verified installer.
