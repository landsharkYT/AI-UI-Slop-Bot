# Never execute scanned repositories in V1

Status: Accepted

AI UI Slop Bot will treat target source and pull requests as untrusted data: it will not install dependencies, run scripts, build applications, import project modules, or execute configuration. GitHub templates will use the read-only `pull_request` event rather than privileged `pull_request_target` processing. This sacrifices some dynamic class and framework resolution, but makes local and GitHub Action scans safer and deterministic; unresolved behavior is reported as reduced analysis coverage instead of being guessed.
