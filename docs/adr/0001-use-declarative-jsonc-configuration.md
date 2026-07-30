# Use declarative JSONC configuration

Status: Accepted

AI UI Slop Bot will use a schema-validated `ai-ui-slop.config.jsonc` file instead of executable TypeScript configuration. JSONC retains comments and editor support while allowing local and GitHub Action scans to read pull-request configuration without executing repository code; the public schema also provides a versioned contract for validation and migration.
