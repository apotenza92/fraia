#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

project_dir="${1:-}"
cleanup_dir=""

if [[ -z "$project_dir" ]]; then
  cleanup_dir="$(mktemp -d "${TMPDIR:-/tmp}/fraia-smoke.XXXXXX")"
  project_dir="$cleanup_dir/demo-project"
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

echo "==> Fraia smoke test project: $project_dir"

echo "==> Running demo flow"
cargo run -p fraia-cli -- demo "$project_dir"

opt_run_dir="$(latest_dir_matching '*')"
if [[ -z "$opt_run_dir" ]]; then
  echo "No run directories were created by demo flow" >&2
  exit 1
fi
assert_file "$project_dir/fraia.project.json"
assert_file "$opt_run_dir/run.json"
assert_file "$opt_run_dir/options.json"
assert_file "$opt_run_dir/snapshot.json"
assert_file "$opt_run_dir/diagnostics.json"
assert_file "$opt_run_dir/summary.md"

echo "==> Adopting option 1 from the latest optimization run"
cargo run -p fraia-cli -- adopt "$project_dir" 1
assert_file "$project_dir/fraia.project.json"

echo "==> Validating adopted authored model"
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
assert_file "$validate_run_dir/member-actions.csv"
assert_file "$validate_run_dir/support-reactions.csv"
assert_file "$validate_run_dir/check-results.csv"
assert_file "$validate_run_dir/summary.md"

echo "==> Re-adopting option 1 after validation to prove validation runs do not hide optimization runs"
cargo run -p fraia-cli -- adopt "$project_dir" 1
assert_file "$project_dir/fraia.project.json"

echo "==> Re-validating after re-adoption"
cargo run -p fraia-cli -- validate "$project_dir"

latest_validate_run_dir="$(latest_dir_matching 'validate-*')"
assert_file "$latest_validate_run_dir/summary.md"

echo "==> Smoke test passed"
echo "Project dir: $project_dir"
if [[ -n "$cleanup_dir" && "${FRAIA_SMOKE_KEEP:-0}" != "1" ]]; then
  echo "Temporary project removed on exit. Set FRAIA_SMOKE_KEEP=1 to retain it."
fi
