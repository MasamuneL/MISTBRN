#!/usr/bin/env bash
# Fetch the pinned Horizon mod SDK API source into mod/.horizon/sdk (git-ignored).
# Verifies SHA256 against the release's SHA256SUMS. The release signature itself is checked by
# the launcher during native-build, not here.
source "$(dirname "$0")/lib.sh"

SDK_DIR="$REPO_ROOT/mod/.horizon/sdk"
if [[ -f "$SDK_DIR/build" && "$(cat "$SDK_DIR/build")" == "$HORIZON_COMMIT" ]]; then
  echo "SDK $HORIZON_COMMIT already present"; exit 0
fi
BASE="${HORIZON_SDK_BASE:-https://github.com/horizon-rust/horizon/releases/download}/sdk-$HORIZON_COMMIT"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
for f in SHA256SUMS sdk.json surface.json api-source.tar.xz; do
  curl -fsSL --retry 3 -o "$TMP/$f" "$BASE/$f" || die "download failed: $f"
done
for f in sdk.json surface.json api-source.tar.xz; do
  expected="$(awk -v n="$f" '$2==n {print $1}' "$TMP/SHA256SUMS")"
  [[ -n "$expected" ]] || die "$f missing from SHA256SUMS"
  [[ "$(sha256 "$TMP/$f")" == "$expected" ]] || die "SHA256 mismatch: $f"
done
grep -q "\"commit\": *\"$HORIZON_COMMIT\"" "$TMP/sdk.json" || die "sdk.json names another build"
rm -rf "$SDK_DIR"; mkdir -p "$SDK_DIR/api"
tar -xJf "$TMP/api-source.tar.xz" -C "$SDK_DIR/api"
cp "$TMP/sdk.json" "$TMP/surface.json" "$SDK_DIR/"
# Written last: a partial fetch is redone by the next run.
echo "$HORIZON_COMMIT" > "$SDK_DIR/build"
echo "SDK $HORIZON_COMMIT fetched into $SDK_DIR"
