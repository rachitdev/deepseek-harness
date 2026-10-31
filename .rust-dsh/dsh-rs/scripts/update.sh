#!/usr/bin/env bash
# Update dsh-rs and its cordis kernel from GitHub, gated on the test suite.
#
#   scripts/update.sh kernel check            show pinned vs upstream kernel rev (read-only)
#   scripts/update.sh kernel apply [REV]      pin the kernel to REV (default: upstream HEAD), run tests, roll back on failure
#   scripts/update.sh self check              show whether origin has commits not in this branch (fetches only)
#   scripts/update.sh self apply              merge origin's default branch, run tests, roll back on failure
#
# Never pushes. Both `apply` modes refuse a dirty working tree and restore the
# tracked files (manifest/lockfile, or HEAD) when tests fail or the run is
# interrupted by Ctrl-C/SIGTERM. Files a failing test run creates are NOT
# removed (they are ignored build output in this repo); SIGKILL is not trapped.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
KERNEL_URL="${KERNEL_URL:-https://github.com/jincdream/cordis-rs}"
COMPAT_TOML="crates/cordis-compat/Cargo.toml"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"

die() { echo "update: $*" >&2; exit 1; }
clean_tree() { [ -z "$(git status --porcelain)" ] || die "working tree is dirty; commit or stash first"; }
pinned_rev() { sed -n 's/.*cordis-rs-core.*rev = "\([0-9a-f]\{40\}\)".*/\1/p' "$COMPAT_TOML"; }
run_tests() { echo "update: running cargo test ..."; cargo test --quiet; }

# Rollback on any unsuccessful exit, including Ctrl-C and SIGTERM mid-test.
# ROLLBACK is set to a command string once a change has been made and cleared
# when the update is verified. SIGKILL cannot be trapped: recover with
# `git status` / `git reset --hard <printed original HEAD>`.
ROLLBACK=""
on_exit() {
  local code=$?
  if [ -n "$ROLLBACK" ]; then
    echo "update: interrupted or failed; restoring original state" >&2
    eval "$ROLLBACK" || echo "update: RESTORE FAILED; original HEAD was ${ORIGINAL_HEAD:-unknown}" >&2
  fi
  exit "$code"
}
trap on_exit EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
ORIGINAL_HEAD="$(git rev-parse HEAD)"; echo "update: starting at $ORIGINAL_HEAD"

kernel() {
  local mode="${1:-check}" want="${2:-}" pinned upstream
  pinned="$(pinned_rev)"; [ -n "$pinned" ] || die "cannot read pinned rev from $COMPAT_TOML"
  upstream="$(git ls-remote "$KERNEL_URL" HEAD | cut -f1)"; [ -n "$upstream" ] || die "cannot reach $KERNEL_URL"
  want="${want:-$upstream}"
  echo "kernel: pinned   $pinned"
  echo "kernel: upstream $upstream"
  [ "$mode" = check ] && { [ "$pinned" = "$upstream" ] && echo "kernel: up to date" || echo "kernel: update available"; return 0; }
  [ "$mode" = apply ] || die "unknown mode: $mode"
  [[ "$want" =~ ^[0-9a-f]{40}$ ]] || die "REV must be a full 40-character commit id"
  [ "$want" != "$pinned" ] || { echo "kernel: already pinned to $want"; return 0; }
  clean_tree
  ROLLBACK="git checkout -- $COMPAT_TOML Cargo.lock"
  echo "kernel: pinning $want (was $pinned)"
  python3 - "$COMPAT_TOML" "$pinned" "$want" <<'PY'
import sys
path, old, new = sys.argv[1:]
text = open(path).read()
assert text.count(old) == 1, "pinned rev must appear exactly once"
open(path, "w").write(text.replace(old, new))
PY
  if cargo update -p cordis-rs-core --precise "$want" >/dev/null 2>&1 || cargo fetch --quiet; then :; fi
  if run_tests; then
    ROLLBACK=""
    echo "kernel: tests pass at $want; review and commit: git commit -am 'Bump cordis-rs kernel to ${want:0:7}'"
  else
    echo "kernel: tests FAILED at $want" >&2
    exit 1
  fi
}

self() {
  local mode="${1:-check}" branch remote_head behind
  git remote get-url origin >/dev/null || die "no origin remote"
  git fetch --quiet origin
  remote_head="$(git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null || true)"
  [ -n "$remote_head" ] || remote_head="origin/$(git ls-remote --symref origin HEAD | sed -n 's#^ref: refs/heads/\([^\t]*\)\tHEAD#\1#p')"
  branch="$(git branch --show-current)"
  behind="$(git rev-list --count "HEAD..$remote_head")"
  echo "self: branch $branch, upstream $remote_head, upstream commits not merged: $behind"
  [ "$mode" = check ] && return 0
  [ "$mode" = apply ] || die "unknown mode: $mode"
  [ "$behind" -gt 0 ] || { echo "self: up to date"; return 0; }
  clean_tree
  local before; before="$(git rev-parse HEAD)"
  ROLLBACK="git merge --abort 2>/dev/null; git reset --hard $before"
  if ! git -c user.name="dsh-update" -c user.email="dsh-update@localhost" merge --no-edit "$remote_head"; then
    git merge --abort || true
    die "merge conflicts with $remote_head; resolve manually (tree restored to $before)"
  fi
  if run_tests; then
    ROLLBACK=""
    echo "self: merged $remote_head, tests pass"
  else
    echo "self: tests FAILED after merge" >&2
    exit 1
  fi
}

case "${1:-}" in
  kernel) shift; kernel "$@" ;;
  self)   shift; self "$@" ;;
  *) sed -n '2,10p' "${BASH_SOURCE[0]}"; exit 2 ;;
esac
