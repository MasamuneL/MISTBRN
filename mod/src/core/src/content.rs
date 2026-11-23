//! Content IDs and balance rules shared by the game adapter and tests.
use crate::{Metal, Pair};

pub const EMPTY_VIAL: &str = "mistbrn:vial_empty";
/// Unlock items: each one unlocks both metals of its pair when its use finishes. Provisional
/// for the V1 demo; P5/P6 progression replaces how they are obtained (docs/plan.md).
pub const UNLOCK_ITEMS: [(&str, Pair); 1] = [("mistbrn:primer_iron_steel", Pair::IronSteel)];
pub const VIAL_MILLI: i64 = 25_000;
pub const FILLED_VIALS: [(&str, Metal); 8] = [
    ("mistbrn:vial_iron", Metal::Iron),
    ("mistbrn:vial_steel", Metal::Steel),
    ("mistbrn:vial_tin", Metal::Tin),
    ("mistbrn:vial_pewter", Metal::Pewter),
    ("mistbrn:vial_zinc", Metal::Zinc),
    ("mistbrn:vial_brass", Metal::Brass),
    ("mistbrn:vial_copper", Metal::Copper),
    ("mistbrn:vial_bronze", Metal::Bronze),
];

pub fn metal_from_vial(id: &str) -> Option<Metal> {
    FILLED_VIALS
        .iter()
        .find_map(|&(vial, metal)| (vial == id).then_some(metal))
}

/// Stable across sessions and platforms, including negative coordinates.
fn location_hash(dimension: &str, pos: [i32; 3]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in dimension
        .bytes()
        .chain(pos.into_iter().flat_map(i32::to_le_bytes))
    {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
    }
    // Avalanche before taking a small modulus so neighbouring coordinates do not form stripes.
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51_afd7_ed55_8ccd);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    hash ^ (hash >> 33)
}

/// Returns the expected vanilla drop and one bonus sample for an eligible ore position.
pub fn ore_sample(
    ore: &str,
    dimension: &str,
    pos: [i32; 3],
) -> Option<(&'static str, &'static str)> {
    let (vanilla, sample, one_in) = match ore.strip_prefix("deepslate_").unwrap_or(ore) {
        "copper_ore" => ("minecraft:raw_copper", "mistbrn:raw_tin", 4),
        "iron_ore" => ("minecraft:raw_iron", "mistbrn:raw_zinc", 3),
        _ => return None,
    };
    location_hash(dimension, pos)
        .is_multiple_of(one_in)
        .then_some((vanilla, sample))
}

/// How many samples an eligible ore gives with a Fortune level: vanilla's ore-drop bonus
/// (`ApplyBonusCount.oreDrops`), one plus `max(0, r - 1)` for `r` in `0..=fortune + 1`, drawn
/// from the position so the same block always gives the same count. Fortune 0 gives 1,
/// Fortune III gives 1 to 4 (2.2 on average). Levels above 3 count as 3.
pub fn sample_count(fortune: u8, dimension: &str, pos: [i32; 3]) -> u8 {
    let fortune = fortune.min(3);
    let draw = location_hash(&format!("{dimension}#fortune"), pos) % (u64::from(fortune) + 2);
    1 + draw.saturating_sub(1) as u8
}

/// Optional discovery on the first opening of selected vanilla structure containers.
pub fn chest_sample(table: &str, dimension: &str, pos: [i32; 3]) -> Option<&'static str> {
    if !matches!(
        table,
        "minecraft:chests/simple_dungeon"
            | "minecraft:chests/abandoned_mineshaft"
            | "minecraft:chests/underwater_ruin_small"
            | "minecraft:chests/underwater_ruin_big"
    ) {
        return None;
    }
    let hash = location_hash(dimension, pos);
    if !hash.is_multiple_of(8) {
        return None;
    }
    Some(["mistbrn:vial_tin", "mistbrn:vial_zinc", "mistbrn:vial_iron"][(hash / 8 % 3) as usize])
}
