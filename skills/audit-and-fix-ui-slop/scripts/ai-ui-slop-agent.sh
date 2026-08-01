#!/usr/bin/env bash
set -uo pipefail

usage() {
  printf '%s\n' \
    'Usage: ai-ui-slop-agent.sh doctor [REPO]' \
    '       ai-ui-slop-agent.sh init [REPO]' \
    '       ai-ui-slop-agent.sh scan [REPO]' \
    '' \
    'Environment:' \
    '  AI_UI_SLOP_BIN       Exact scanner binary; otherwise use ai-ui-slop on PATH.' \
    '  AI_UI_SLOP_PROGRESS  auto, always, or never (default: always).' \
    '  AI_UI_SLOP_JOBS      Optional positive worker count.' \
    '  AI_UI_SLOP_RUN_ROOT  Evidence directory (default: REPO/.ai-ui-slop/agent-runs).'
}

fail() {
  printf 'agent audit: %s\n' "$1" >&2
  exit "${2:-2}"
}

resolve_binary() {
  if [[ -n "${AI_UI_SLOP_BIN:-}" ]]; then
    [[ -x "$AI_UI_SLOP_BIN" ]] || fail "AI_UI_SLOP_BIN is not executable: $AI_UI_SLOP_BIN" 127
    printf '%s\n' "$AI_UI_SLOP_BIN"
    return
  fi
  command -v ai-ui-slop 2>/dev/null || fail 'ai-ui-slop is not on PATH; set AI_UI_SLOP_BIN' 127
}

resolve_repository() {
  local requested=${1:-.}
  [[ -d "$requested" ]] || fail "repository directory does not exist: $requested"
  (cd "$requested" && pwd -P) || fail "cannot resolve repository: $requested"
}

write_exit_code() {
  local path=$1
  local code=$2
  printf '%s\n' "$code" > "$path"
}

command_name=${1:-}
[[ -n "$command_name" ]] || { usage >&2; exit 2; }
case "$command_name" in
  doctor|init|scan) ;;
  -h|--help|help) usage; exit 0 ;;
  *) usage >&2; fail "unknown command: $command_name" ;;
esac
shift
[[ $# -le 1 ]] || fail 'too many arguments'

repository=$(resolve_repository "${1:-.}")
scanner=$(resolve_binary)

case "$command_name" in
  doctor)
    "$scanner" version || exit $?
    printf 'repository: %s\n' "$repository"
    if [[ -f "$repository/ai-ui-slop.config.jsonc" ]]; then
      printf '%s\n' 'configuration: present'
      "$scanner" config validate "$repository" --effective default
    else
      printf '%s\n' 'configuration: absent; run init only with permission'
      exit 2
    fi
    ;;
  init)
    if [[ -e "$repository/ai-ui-slop.config.jsonc" ]]; then
      fail 'configuration already exists; refusing to overwrite it'
    fi
    "$scanner" init "$repository"
    ;;
  scan)
    [[ -f "$repository/ai-ui-slop.config.jsonc" ]] || fail 'configuration is absent; review and run init first'
    progress=${AI_UI_SLOP_PROGRESS:-always}
    case "$progress" in auto|always|never) ;; *) fail 'AI_UI_SLOP_PROGRESS must be auto, always, or never' ;; esac
    if [[ -n "${AI_UI_SLOP_JOBS:-}" && ! "$AI_UI_SLOP_JOBS" =~ ^[1-9][0-9]*$ ]]; then
      fail 'AI_UI_SLOP_JOBS must be a positive integer'
    fi

    run_root=${AI_UI_SLOP_RUN_ROOT:-"$repository/.ai-ui-slop/agent-runs"}
    mkdir -p "$run_root" || fail "cannot create run root: $run_root" 4
    run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"
    run_directory="$run_root/$run_id"
    mkdir "$run_directory" || fail "cannot create run directory: $run_directory" 4

    "$scanner" version > "$run_directory/version.txt" 2>&1
    validation_code=0
    "$scanner" config validate "$repository" --effective default \
      > "$run_directory/config-validation.stdout" \
      2> "$run_directory/config-validation.stderr" || validation_code=$?
    write_exit_code "$run_directory/config-validation-exit-code.txt" "$validation_code"
    if [[ $validation_code -ne 0 ]]; then
      write_exit_code "$run_directory/exit-code.txt" "$validation_code"
      printf 'agent audit run: %s\n' "$run_directory"
      exit "$validation_code"
    fi

    scan_arguments=(scan "$repository" --format json --progress "$progress")
    if [[ -n "${AI_UI_SLOP_JOBS:-}" ]]; then
      scan_arguments+=(--jobs "$AI_UI_SLOP_JOBS")
    fi
    scan_code=0
    "$scanner" "${scan_arguments[@]}" \
      > "$run_directory/scan.stdout.json" \
      2> "$run_directory/scan.stderr" || scan_code=$?
    write_exit_code "$run_directory/exit-code.txt" "$scan_code"

    canonical="$repository/.ai-ui-slop/reports"
    [[ -f "$canonical/report.json" ]] && cp "$canonical/report.json" "$run_directory/report.json"
    [[ -f "$canonical/refactoring-brief.md" ]] && cp "$canonical/refactoring-brief.md" "$run_directory/refactoring-brief.md"

    printf 'agent audit run: %s\n' "$run_directory"
    printf 'scanner exit code: %s\n' "$scan_code"
    exit "$scan_code"
    ;;
esac
