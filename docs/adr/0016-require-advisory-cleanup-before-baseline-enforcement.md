# Require advisory cleanup before baseline enforcement

Status: Accepted

Each Analysis Scope must progress from an unapproved `init` draft through effective-policy inspection, a repository-wide advisory scan, Design Authority review, and incremental cleanup before candidate creation and explicit baseline acceptance; acceptance binds approval to the exact effective-policy fingerprint, after which enforcement may be enabled deliberately. Missing lifecycle prerequisites are invalid configuration, while coverage loss during otherwise valid enforcement is insufficient analysis. The GitHub Action defaults to advisory, and no command may automatically approve assumptions, suppress Findings, accept debt, or label the initial repository state clean.
