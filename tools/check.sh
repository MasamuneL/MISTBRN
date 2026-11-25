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

echo "== content: JSON, item assets, translations and recipe IDs"
require jq od
assets=mod/assets/mistbrn
data=mod/data/mistbrn
while IFS= read -r -d '' path; do
  jq -se 'length == 1 and (.[0] | type == "object")' "$path" >/dev/null ||
    die "invalid content JSON: $path"
done < <(find "$assets" "$data" -type f -name '*.json' -print0)
keys="$(jq -c 'keys' "$assets/lang/en_us.json")"
[[ "$keys" == "$(jq -c 'keys' "$assets/lang/es_es.json")" ]] ||
  die "en_us and es_es must contain the same keys"

for definition in "$data"/horizon/item/*.json; do
  id="${definition##*/}"; id="${id%.json}"
  [[ -f "$definition" ]] || die "no registered items"
  jq -e --arg model "mistbrn:item/$id" '.model.model == $model' "$assets/items/$id.json" >/dev/null ||
    die "missing item model for $id"
  jq -e --arg texture "mistbrn:item/$id" '.textures.layer0 == $texture' "$assets/models/item/$id.json" >/dev/null ||
    die "missing model texture for $id"
  png="$assets/textures/item/$id.png"
  [[ -s "$png" ]] || die "missing texture for $id"
  header="$(od -An -tx1 -N24 "$png" | tr -d '[:space:]')"
  [[ "$header" == 89504e470d0a1a0a0000000d494844520000001000000010 ]] ||
    die "texture for $id must be a 16x16 PNG"
  (( $(wc -c < "$png") <= MAX_ASSET_PNG_BYTES )) || die "texture too large: $png"
  for language in en_us es_es; do
    jq -e --arg key "item.mistbrn.$id" '.[$key] | type == "string" and length > 0' \
      "$assets/lang/$language.json" >/dev/null || die "missing $language translation for $id"
  done
done

# Every item that names a mod equipment asset has that asset file.
for definition in "$data"/horizon/item/*.json; do
  # The whole value is checked inside jq: any present asset_id must be a namespaced id string.
  asset="$(jq -er '(.components["minecraft:equippable"] // {}) |
    if has("asset_id") | not then "-"
    elif (.asset_id | type == "string" and test("\\A(mistbrn|minecraft):[a-z0-9_]+\\z")) then .asset_id
    else error("asset_id must be mistbrn:<id> or minecraft:<id>") end' "$definition")" ||
    die "invalid equipment asset_id in $definition"
  if [[ "$asset" == mistbrn:* ]]; then
    [[ -f "$assets/equipment/${asset#mistbrn:}.json" ]] || die "missing equipment asset $asset"
  fi
done

for equipment in "$assets"/equipment/*.json; do
  [[ -f "$equipment" ]] || continue
  textures="$(jq -er '.layers.humanoid | if type == "array" and length > 0 then .[].texture else error("no humanoid layers") end' "$equipment")" ||
    die "equipment needs humanoid layers with textures: $equipment"
  while IFS= read -r texture; do
    png="$assets/textures/entity/equipment/humanoid/${texture#mistbrn:}.png"
    [[ "$texture" == mistbrn:* && -s "$png" ]] || die "missing equipment texture $texture for $equipment"
  done <<< "$textures"
done

# Every block (P3c) has its blockstate, model, block item model, textures, loot and name.
for definition in "$data"/horizon/block/*.json; do
  [[ -f "$definition" ]] || continue
  id="${definition##*/}"; id="${id%.json}"
  jq -e --arg model "mistbrn:block/$id" '.variants[""].model == $model' \
    "$assets/blockstates/$id.json" >/dev/null || die "missing blockstate for block $id"
  jq -e --arg model "mistbrn:block/$id" '.model.model == $model' "$assets/items/$id.json" >/dev/null ||
    die "missing block item model for $id"
  textures="$(jq -er '.textures | if type == "object" and length > 0 then .[] else error("no textures") end' \
    "$assets/models/block/$id.json")" || die "missing block model textures for $id"
  while IFS= read -r texture; do
    png="$assets/textures/${texture#mistbrn:}.png"
    [[ "$texture" == mistbrn:block/* && -s "$png" ]] || die "missing texture $texture for block $id"
    header="$(od -An -tx1 -N24 "$png" | tr -d '[:space:]')"
    [[ "$header" == 89504e470d0a1a0a0000000d494844520000001000000010 ]] ||
      die "texture for block $id must be a 16x16 PNG"
  done <<< "$textures"
  [[ -f "$data/loot_table/blocks/$id.json" ]] || die "missing loot table for block $id"
  for language in en_us es_es; do
    jq -e --arg key "block.mistbrn.$id" '.[$key] | type == "string" and length > 0' \
      "$assets/lang/$language.json" >/dev/null || die "missing $language translation for block $id"
  done
done

for recipe in "$data"/recipe/*.json; do
  ids="$(jq -er '
    if (.result.id | type) != "string" then error("missing recipe result ID") else
      [.key[]?, .ingredients[]?, .ingredient?, .result.id] | map(select(. != null)) | flatten | .[]
    end
  ' "$recipe")"
  while IFS= read -r id; do
    case "$id" in
      mistbrn:*) [[ -f "$data/horizon/item/${id#mistbrn:}.json" || -f "$data/horizon/block/${id#mistbrn:}.json" ]] ||
        die "unknown item $id in $recipe" ;;
      minecraft:glass|minecraft:iron_ingot|minecraft:copper_ingot|minecraft:coal|minecraft:book) ;;
      minecraft:cauldron|minecraft:blaze_powder) ;;
      minecraft:copper_nugget|minecraft:gold_nugget|minecraft:iron_nugget|minecraft:cobblestone|'#minecraft:stone_crafting_materials'|minecraft:stick|minecraft:string|minecraft:gray_wool|'#minecraft:logs') ;;
      *) die "unknown item $id in $recipe" ;;
    esac
  done <<< "$ids"
done

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
# Sizes of the staged blobs (the index), not of the working tree.
files="$(git ls-files --cached)"
sizes="$(git ls-files --cached -s | awk '{print $2}' | git cat-file --batch-check='%(objectsize)')"
total=0; bad=0
while IFS=$'\t' read -r size f; do
  total=$((total + size))
  if (( size > MAX_FILE_BYTES )); then echo "too large (${size} B > ${MAX_FILE_BYTES}): $f"; bad=1; fi
  if [[ "$f" =~ $FORBIDDEN ]]; then echo "forbidden file type: $f"; bad=1; fi
  if [[ "$f" == *.png ]]; then
    if [[ "$f" != mod/assets/mistbrn/textures/* ]]; then echo "png outside mod textures: $f"; bad=1
    elif (( size > MAX_ASSET_PNG_BYTES )); then echo "texture too large (${size} B): $f"; bad=1; fi
  fi
done < <(paste <(printf '%s\n' "$sizes") <(printf '%s\n' "$files"))
echo "tree: $total bytes in $(wc -l <<< "$files" | tr -d ' ') files (limit $MAX_TREE_BYTES)"
(( total <= MAX_TREE_BYTES )) || { echo "tree exceeds limit"; bad=1; }
(( bad == 0 )) || die "repository weight check failed"
echo "OK"
