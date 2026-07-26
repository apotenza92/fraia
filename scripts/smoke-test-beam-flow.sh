#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

project_dir="${1:-}"
cleanup_dir=""

if [[ -z "$project_dir" ]]; then
  cleanup_dir="$(mktemp -d "${TMPDIR:-/tmp}/fraia-beam-smoke.XXXXXX")"
  project_dir="$cleanup_dir/beam-project"
fi

cleanup() {
  if [[ -n "$cleanup_dir" && "${FRAIA_SMOKE_KEEP:-0}" != "1" ]]; then
    rm -rf "$cleanup_dir"
  fi
}
trap cleanup EXIT

assert_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "Missing expected file: $path" >&2
    exit 1
  fi
}

latest_dir_matching() {
  local pattern="$1"
  find "$project_dir/runs" -mindepth 1 -maxdepth 1 -type d -name "$pattern" -print | sort | tail -n 1
}

echo "==> Fraia beam smoke test project: $project_dir"

echo "==> Creating beam demo project"
cargo run -p fraia-cli -- beam-demo "$project_dir"
assert_file "$project_dir/fraia.project.json"

echo "==> Sizing beam"
cargo run -p fraia-cli -- beam-size "$project_dir"
beam_run_dir="$(latest_dir_matching 'beam-size-*')"
if [[ -z "$beam_run_dir" ]]; then
  echo "No beam sizing run directory was created" >&2
  exit 1
fi
assert_file "$beam_run_dir/sizing.json"
assert_file "$beam_run_dir/summary.md"

echo "==> Validating sized beam project"
cargo run -p fraia-cli -- validate "$project_dir"
validate_run_dir="$(latest_dir_matching 'validate-*')"
if [[ -z "$validate_run_dir" ]]; then
  echo "No validation run directory was created" >&2
  exit 1
fi
assert_file "$validate_run_dir/validation.json"
assert_file "$validate_run_dir/realization.json"
assert_file "$validate_run_dir/design-actions.json"
assert_file "$validate_run_dir/checks.json"
assert_file "$validate_run_dir/summary.md"

echo "==> Beam smoke test passed"
echo "Project dir: $project_dir"
if [[ -n "$cleanup_dir" && "${FRAIA_SMOKE_KEEP:-0}" != "1" ]]; then
  echo "Temporary project removed on exit. Set FRAIA_SMOKE_KEEP=1 to retain it."
fi
