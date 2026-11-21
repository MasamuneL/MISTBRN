#!/usr/bin/env bash
# Build the mod with the Horizon launcher for the dev instance, from a clean staged copy.
# Usage: tools/build-native.sh <build-name>   (a new name per build; existing names are refused)
source "$(dirname "$0")/lib.sh"

name="${1:-}"; [[ "$name" =~ ^[a-z0-9][a-z0-9-]{0,40}$ ]] || die "usage: $0 <build-name> ([a-z0-9-])"
RT="$(runtime_root)"
stage="$RT/staged/$name"; logdir="$RT/logs/$name"
[[ ! -e "$stage" && ! -e "$logdir" ]] || die "build $name already exists: inspect $stage / $logdir"
"$REPO_ROOT/tools/fetch-sdk.sh" >/dev/null

# Only the package's own inputs: never .horizon/, target/ or a local Cargo.lock.
pkg="$stage/mistbrn"; mkdir -p "$pkg" "$logdir"
( cd "$REPO_ROOT/mod" && for p in horizon-mod.toml Cargo.toml src assets data; do
    cp -R "$p" "$pkg/"; done )
find "$pkg" -name target -type d -prune -exec rm -rf {} + -o -name Cargo.lock -delete -o -name .DS_Store -delete

receipt="$stage/receipt.txt"
{
  echo "build=$name"
  echo "git_head=$(git -C "$REPO_ROOT" rev-parse HEAD)"
  echo "git_dirty=$([[ -n "$(git -C "$REPO_ROOT" status --porcelain -- mod)" ]] && echo yes || echo no)"
  echo "horizon=$HORIZON_COMMIT os=$(os_kind) arch=$(uname -m)"
  ( cd "$pkg" && find . -type f | LC_ALL=C sort | while read -r f; do echo "input $(sha256 "$f") $f"; done )
} > "$receipt"

set +e
"$(launcher)" native-build "$stage" --instance "$DEV_INSTANCE" --version "$HORIZON_VERSION" \
  --trust-native-code 2>&1 | tee "$logdir/build.log"
code=${PIPESTATUS[0]}
set -e
echo "exit_code=$code" >> "$receipt"
echo "receipt: $receipt"
exit "$code"
