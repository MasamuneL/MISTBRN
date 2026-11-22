//! Which blocks and items an iron pull or a steel push can reach (P4).
//!
//! The pinned SDK reads no block or item tags, so the list is explicit: exact ids plus a few
//! naming rules over vanilla ids (`iron`, `gold`, `copper`, `netherite_`, `chainmail_`,
//! `_bucket`). Food, spawn eggs and smithing templates whose names mention a metal are not metal.

/// Vanilla ids that hold metal without naming one.
const VANILLA_EXACT: [&str; 26] = [
    "anvil",
    "chipped_anvil",
    "damaged_anvil",
    "cauldron",
    "water_cauldron",
    "lava_cauldron",
    "powder_snow_cauldron",
    "hopper",
    "bell",
    "lantern",
    "soul_lantern",
    "chain",
    "rail",
    "detector_rail",
    "powered_rail",
    "activator_rail",
    "heavy_weighted_pressure_plate",
    "light_weighted_pressure_plate",
    "lightning_rod",
    "bucket",
    "shears",
    "compass",
    "clock",
    "flint_and_steel",
    "minecart",
    "spyglass",
];

/// Vanilla ids that name a metal but are not metal objects.
const VANILLA_EXCLUDED: [&str; 5] = [
    "golden_apple",
    "enchanted_golden_apple",
    "golden_carrot",
    "golden_dandelion",
    "netherite_upgrade_smithing_template",
];

/// The mod's own metal items besides ingots and flakes.
const MOD_EXACT: [&str; 5] = ["raw_tin", "raw_zinc", "clip", "boxing", "bronze_earring"];

/// Whether a block or item id (`namespace:path`) is metal for iron and steel.
pub fn is_metal(id: &str) -> bool {
    match id.split_once(':') {
        Some(("minecraft", path)) => vanilla_metal(path),
        Some(("mistbrn", path)) => {
            path.ends_with("_ingot") || path.ends_with("_flakes") || MOD_EXACT.contains(&path)
        }
        _ => false,
    }
}

fn vanilla_metal(path: &str) -> bool {
    if path.ends_with("_spawn_egg") || VANILLA_EXCLUDED.contains(&path) {
        return false;
    }
    VANILLA_EXACT.contains(&path)
        || path.contains("iron")
        || path.contains("gold")
        || path.contains("copper")
        || path.starts_with("netherite_")
        || path.starts_with("chainmail_")
        || path.ends_with("_bucket")
        || path.ends_with("_minecart")
}

/// The coins: light metal items made to be pushed.
pub fn is_coin(id: &str) -> bool {
    matches!(id, "mistbrn:clip" | "mistbrn:boxing")
}
