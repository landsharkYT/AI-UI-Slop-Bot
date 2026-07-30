# Separate source analysis from safe artifact writes

Status: Accepted

The scanner will be source-non-mutating rather than literally read-only: it never changes target code, pre-existing project configuration, dependencies, Git state, or application state, while explicit commands may create declared scanner artifacts inside the repository boundary. Artifact writes use type-aware refusal, hostile-filesystem checks, same-directory temporary files, validation and atomic replacement, and no force option may overwrite unrelated data. Repository-derived strings are hostile presentation input and must be escaped independently for terminal, JSON, Markdown, hyperlinks, and GitHub summaries; sensitive token payloads are reduced to semantic redactions and are never exposed through raw low-entropy hashes.
