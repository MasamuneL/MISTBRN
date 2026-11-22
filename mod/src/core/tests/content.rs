use allomancy_core::{Metal, metal_from_vial, ore_sample, sample_count};

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

#[test]
fn fortune_raises_sample_counts_like_vanilla_ore_drops() {
    let mut totals = [0u32; 4];
    for x in -50..50 {
        for z in -50..50 {
            let pos = [x, 12, z];
            assert_eq!(sample_count(0, "minecraft:overworld", pos), 1);
            for fortune in 1..=3u8 {
                let n = sample_count(fortune, "minecraft:overworld", pos);
                assert!((1..=fortune + 1).contains(&n), "fortune {fortune}: {n}");
                assert_eq!(n, sample_count(fortune, "minecraft:overworld", pos));
                totals[usize::from(fortune)] += u32::from(n);
            }
            assert_eq!(
                sample_count(9, "minecraft:overworld", pos),
                sample_count(3, "minecraft:overworld", pos)
            );
        }
    }
    // Vanilla means: 4/3, 7/4 and 11/5 over 10 000 positions, within 3 %.
    for (fortune, mean_milli) in [(1, 1333), (2, 1750), (3, 2200)] {
        let mean = totals[fortune] / 10; // thousandths of a sample per block
        assert!(
            mean.abs_diff(mean_milli) <= mean_milli * 3 / 100,
            "fortune {fortune}: {mean}"
        );
    }
}

/// Counts the in-game C6F fixture expects with Fortune III: copper x=5 at z=3 gives 4 tin (1 without
/// Fortune); iron x=0,1,5 at z=9 gives 4+2+3 zinc (3 without Fortune).
#[test]
fn c6_fortune_fixture_counts() {
    let at = |x, z| sample_count(3, "minecraft:overworld", [x, 160, z]);
    assert_eq!(at(5, 3), 4);
    assert_eq!(at(0, 9) + at(1, 9) + at(5, 9), 9);
}

#[test]
#[ignore = "generator for the C6F fixture"]
fn print_iron_rows_for_fortune() {
    for z in [7, 9, 11] {
        let row: Vec<_> = (0..8)
            .filter(|&x| ore_sample("iron_ore", "minecraft:overworld", [x, 160, z]).is_some())
            .map(|x| (x, sample_count(3, "minecraft:overworld", [x, 160, z])))
            .collect();
        println!("z={z} {row:?}");
    }
}
