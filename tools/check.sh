#!/usr/bin/env bash
# Pre-merge gate: formatting, lints, core tests, adapter type-check and repository weight.
# Usage: tools/check.sh            (run from anywhere; exit 0 only if every check passes)
source "$(dirname "$0")/lib.sh"
cd "$REPO_ROOT"

MAX_FILE_BYTES=$((64 * 1024))
MAX_TREE_BYTES=$((1024 * 1024))
FORBIDDEN='\.(zip|jpg|jpeg|gif|log|dat|dat_old|mca|dylib|so|dll|exe|jar|tar|xz|gz)$'
# Original mod textures are the only images allowed, each small.
MAX_ASSET_PNG_BYTES=$((16 * 1024))

echo "== core: fmt, clippy, tests"
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

echo "== adapter: fmt, clippy against the pinned SDK"
tools/fetch-sdk.sh >/dev/null
rt="$(runtime_root)"
# One target dir per checkout: worktrees must never share stale adapter builds.
CARGO_TARGET_DIR="$rt/target-check-$(printf %s "$REPO_ROOT" | cksum | cut -d' ' -f1)"
export CARGO_TARGET_DIR
# Resolve the adapter with the SDK's own lock so its dependency versions are the SDK's, then
# drop the copy: the mod has no registry dependencies of its own and versions no lock.
cp mod/.horizon/sdk/api/Cargo.lock mod/Cargo.lock
trap 'rm -f "$REPO_ROOT/mod/Cargo.lock"' EXIT
( cd mod && cargo fmt -- --check && cargo clippy --all-targets -- -D warnings && cargo test )

echo "== repository weight (tracked + staged files)"
files="$(git ls-files --cached)"
total=0; bad=0
while IFS= read -r f; do
  [[ -f "$f" ]] || continue
  size=$(wc -c < "$f" | tr -d ' ')
  total=$((total + size))
  if (( size > MAX_FILE_BYTES )); then echo "too large (${size} B > ${MAX_FILE_BYTES}): $f"; bad=1; fi
  if [[ "$f" =~ $FORBIDDEN ]]; then echo "forbidden file type: $f"; bad=1; fi
  if [[ "$f" == *.png ]]; then
    if [[ "$f" != mod/assets/mistbrn/textures/* ]]; then echo "png outside mod textures: $f"; bad=1
    elif (( size > MAX_ASSET_PNG_BYTES )); then echo "texture too large (${size} B): $f"; bad=1; fi
  fi
done <<< "$files"
echo "tree: $total bytes in $(wc -l <<< "$files" | tr -d ' ') files (limit $MAX_TREE_BYTES)"
(( total <= MAX_TREE_BYTES )) || { echo "tree exceeds limit"; bad=1; }
(( bad == 0 )) || die "repository weight check failed"
echo "OK"
