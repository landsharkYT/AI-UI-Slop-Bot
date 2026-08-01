#!/usr/bin/env bash
set -uo pipefail

usage() {
  printf '%s\n' \
    'Usage: scripts/cleanup.sh [inspect|routine|full] [OPTIONS]' \
    '' \
    'Modes:' \
    '  inspect  Show generated-artifact sizes and delete nothing (default).' \
    '  routine  Remove debug/test and temporary mutation workspaces; preserve release output.' \
    '  full     Run cargo clean against this repository target; requires --yes.' \
    '' \
    'Options:' \
    '  --dry-run               Print removals without changing the filesystem.' \
    '  --include-qualification  Remove raw qualification and mutants.out* evidence; requires --yes.' \
    '  --include-opencode      Also remove .opencode/node_modules; requires --yes.' \
    '  --yes                   Confirm full or optional cache cleanup.' \
    '  -h, --help              Show this help.'
}

fail() {
  printf 'cleanup: %s\n' "$1" >&2
  exit "${2:-2}"
}

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P) \
  || fail 'cannot resolve scripts directory'
repository=$(cd -- "$script_directory/.." && pwd -P) \
  || fail 'cannot resolve repository root'

[[ "$repository" != "/" ]] || fail 'refusing to operate on the filesystem root'
if [[ -n "${HOME:-}" ]]; then
  home_directory=$(cd -- "$HOME" 2>/dev/null && pwd -P) \
    || fail 'cannot resolve HOME for safety validation'
  [[ "$repository" != "$home_directory" ]] || fail 'refusing to operate on HOME'
fi
[[ -f "$repository/Cargo.toml" ]] || fail 'Cargo.toml is missing from repository root'
[[ -e "$repository/.git" ]] || fail '.git is missing from repository root'

mode=inspect
dry_run=false
include_opencode=false
include_qualification=false
confirmed=false
mode_seen=false

for argument in "$@"; do
  case "$argument" in
    inspect|routine|full)
      [[ "$mode_seen" == false ]] || fail 'only one cleanup mode may be selected'
      mode=$argument
      mode_seen=true
      ;;
    --dry-run) dry_run=true ;;
    --include-qualification) include_qualification=true ;;
    --include-opencode) include_opencode=true ;;
    --yes) confirmed=true ;;
    -h|--help) usage; exit 0 ;;
    *) usage >&2; fail "unknown argument: $argument" ;;
  esac
done

target_directory="$repository/target"
opencode_modules="$repository/.opencode/node_modules"

relative_name() {
  local path=$1
  printf '%s\n' "${path#"$repository/"}"
}

print_size() {
  local path=$1
  local label
  label=$(relative_name "$path")
  if [[ -e "$path" || -L "$path" ]]; then
    local size
    size=$(du -sh -- "$path" 2>/dev/null | awk '{print $1}') \
      || size='unavailable'
    printf '%-30s %s\n' "$label" "$size"
  else
    printf '%-30s %s\n' "$label" 'absent'
  fi
}

inspect() {
  printf 'mode: inspect\nrepository: %s\n' "$repository"
  print_size "$target_directory/debug"
  print_size "$target_directory/release"
  print_size "$target_directory/qualification"
  print_size "$target_directory"
  print_size "$repository/mutants.out"
  print_size "$opencode_modules"
  printf '%s\n' 'committed evidence and .ai-ui-slop reports are never cleanup targets'
}

validate_candidate() {
  local path=$1
  [[ "$path" == "$repository/"* ]] || fail "candidate escapes repository: $path"
  [[ "$path" != "$repository" ]] || fail 'refusing to remove repository root'
  local current=$path
  while [[ "$current" != "$repository" ]]; do
    [[ ! -L "$current" ]] \
      || fail "cleanup target or ancestor is a symbolic link: $(relative_name "$current")"
    current=$(dirname -- "$current")
  done
}

remove_candidate() {
  local path=$1
  [[ -e "$path" ]] || return
  validate_candidate "$path"
  local label
  label=$(relative_name "$path")
  if [[ "$dry_run" == true ]]; then
    printf 'would remove: %s\n' "$label"
  else
    rm -rf -- "$path" || fail "failed to remove: $label" 4
    printf 'removed: %s\n' "$label"
  fi
}

if [[ "$mode" == inspect ]]; then
  inspect
  exit 0
fi

[[ ! -L "$target_directory" ]] \
  || fail 'target directory is a symbolic link; refusing cleanup'

if [[ "$mode" == full && "$dry_run" == false && "$confirmed" == false ]]; then
  fail 'full cleanup requires --yes'
fi
if [[ "$include_opencode" == true && "$dry_run" == false && "$confirmed" == false ]]; then
  fail '--include-opencode requires --yes'
fi
if [[ "$include_qualification" == true && "$dry_run" == false && "$confirmed" == false ]]; then
  fail '--include-qualification requires --yes'
fi

if [[ "$mode" == routine ]]; then
  candidates=(
    "$target_directory/debug"
  )
  shopt -s nullglob
  mutation_candidates=(
    "$target_directory"/mutants-tmp
    "$target_directory"/mutants-tmp-*
  )
  shopt -u nullglob
  candidates+=("${mutation_candidates[@]}")
  if [[ "$include_qualification" == true ]]; then
    shopt -s nullglob
    qualification_candidates=(
      "$target_directory/qualification"
      "$repository"/mutants.out*
    )
    shopt -u nullglob
    candidates+=("${qualification_candidates[@]}")
  fi
  if [[ "$include_opencode" == true ]]; then
    candidates+=("$opencode_modules")
  fi
  for candidate in "${candidates[@]}"; do
    validate_candidate "$candidate"
  done
  for candidate in "${candidates[@]}"; do
    remove_candidate "$candidate"
  done
  printf '%s\n' 'preserved: target/release'
  if [[ "$include_qualification" == false ]]; then
    printf '%s\n' 'preserved: target/qualification and mutants.out*'
  fi
  printf '%s\n' 'preserved: docs/evidence and .ai-ui-slop reports'
else
  validate_candidate "$target_directory"
  if [[ "$include_opencode" == true ]]; then
    validate_candidate "$opencode_modules"
  fi
  root_qualification_candidates=()
  if [[ "$include_qualification" == true ]]; then
    shopt -s nullglob
    root_qualification_candidates=(
      "$repository"/mutants.out*
    )
    shopt -u nullglob
    for candidate in "${root_qualification_candidates[@]}"; do
      validate_candidate "$candidate"
    done
  fi
  cargo_arguments=(
    clean
    --manifest-path "$repository/Cargo.toml"
    --target-dir "$target_directory"
  )
  if [[ "$dry_run" == true ]]; then
    cargo_arguments+=(--dry-run)
  fi
  cargo "${cargo_arguments[@]}" || exit $?
  if [[ "$include_opencode" == true ]]; then
    remove_candidate "$opencode_modules"
  fi
  for candidate in "${root_qualification_candidates[@]}"; do
    remove_candidate "$candidate"
  done
  printf '%s\n' 'preserved: docs/evidence and .ai-ui-slop reports'
fi
