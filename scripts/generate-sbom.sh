#!/usr/bin/env bash
set -euo pipefail

destination="${1:-ai-ui-slop.spdx.json}"
python3 "$(dirname "$0")/generate-sbom.py" "$destination"
