use allomancy_core::{Metal, metal_from_vial, ore_sample};

#[test]
fn only_filled_vials_map_to_metals() {
    let ids = [
        "iron", "steel", "tin", "pewter", "zinc", "brass", "copper", "bronze",
    ];
    for (name, metal) in ids.into_iter().zip(Metal::ALL) {
        assert_eq!(
            metal_from_vial(&format!("mistbrn:vial_{name}")),
            Some(metal)
        );
    }
    assert_eq!(metal_from_vial("mistbrn:vial_empty"), None);
    assert_eq!(metal_from_vial("other:vial_iron"), None);
}

#[test]
fn existing_ores_have_deterministic_sample_rates() {
    for (ore, sample, min, max) in [
        ("copper_ore", "mistbrn:raw_tin", 2300, 2700),
        ("iron_ore", "mistbrn:raw_zinc", 3133, 3533),
    ] {
        let mut count = 0;
        for x in -50..50 {
            for z in -50..50 {
                let pos = [x, -32, z];
                let drop = ore_sample(ore, "minecraft:overworld", pos);
                assert_eq!(
                    drop,
                    ore_sample(&format!("deepslate_{ore}"), "minecraft:overworld", pos)
                );
                if let Some((_, drop)) = drop {
                    assert_eq!(drop, sample);
                    count += 1;
                }
            }
        }
        assert!((min..=max).contains(&count), "{ore}: {count}");
    }
    assert_eq!(ore_sample("gold_ore", "minecraft:overworld", [0; 3]), None);
}
