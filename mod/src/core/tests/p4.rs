use allomancy_core::physics::*;
use allomancy_core::{Vec3, is_coin, is_metal};

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3 { x, y, z }
}

fn input(target: Vec3, body: Body, direction: Direction) -> StepInput {
    StepInput {
        player: v(0.0, 0.0, 0.0),
        player_velocity: v(0.0, 0.0, 0.0),
        target,
        target_velocity: v(0.0, 0.0, 0.0),
        body,
        direction,
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

// Catches metal ids missed by the naming rules and food or spawn eggs taken for metal.
#[test]
fn metal_ids() {
    for id in [
        "minecraft:iron_block",
        "minecraft:deepslate_iron_ore",
        "minecraft:iron_bars",
        "minecraft:iron_chain",
        "minecraft:chain",
        "minecraft:anvil",
        "minecraft:water_cauldron",
        "minecraft:hopper",
        "minecraft:rail",
        "minecraft:powered_rail",
        "minecraft:gold_block",
        "minecraft:nether_gold_ore",
        "minecraft:golden_sword",
        "minecraft:waxed_weathered_cut_copper_stairs",
        "minecraft:copper_ingot",
        "minecraft:lightning_rod",
        "minecraft:netherite_block",
        "minecraft:netherite_ingot",
        "minecraft:chainmail_helmet",
        "minecraft:water_bucket",
        "minecraft:chest_minecart",
        "minecraft:iron_nugget",
        "mistbrn:clip",
        "mistbrn:boxing",
        "mistbrn:steel_ingot",
        "mistbrn:zinc_flakes",
        "mistbrn:raw_tin",
    ] {
        assert!(is_metal(id), "{id} should be metal");
    }
    for id in [
        "minecraft:stone",
        "minecraft:glass",
        "minecraft:golden_apple",
        "minecraft:golden_carrot",
        "minecraft:iron_golem_spawn_egg",
        "minecraft:copper_golem_spawn_egg",
        "minecraft:netherite_upgrade_smithing_template",
        "minecraft:oak_planks",
        "mistbrn:glass_dagger",
        "mistbrn:vial_iron",
        "mistbrn:mistcloak",
        "othermod:iron_block",
        "iron_block",
    ] {
        assert!(!is_metal(id), "{id} should not be metal");
    }
    assert!(is_coin("mistbrn:clip") && is_coin("mistbrn:boxing"));
    assert!(!is_coin("minecraft:iron_nugget"));
}

// Catches a wrong falloff: full at the player, zero at and beyond the range.
#[test]
fn acceleration_falls_linearly_to_the_range() {
    let r = ForceRules::BURNING;
    assert!(close(acceleration(&r, 0.0), 0.12));
    assert!(close(acceleration(&r, 8.0), 0.06));
    assert!(close(acceleration(&r, 16.0), 0.0));
    assert!(close(acceleration(&r, 100.0), 0.0));
    assert!(close(acceleration(&r, -1.0), 0.0));
    let f = ForceRules::FLARED;
    assert!(close(acceleration(&f, 12.0), 0.12));
}

// Catches swapped signs: iron draws the player to an anchor, steel drives it away.
#[test]
fn anchor_moves_the_player_toward_or_away() {
    let r = ForceRules::BURNING;
    let pull = step(&r, &input(v(8.0, 0.0, 0.0), Body::Anchor, Direction::Pull)).unwrap();
    assert!(pull.anchored && pull.target_velocity.is_none());
    let vel = pull.player_velocity.unwrap();
    assert!(close(vel.x, 0.06) && close(vel.y, 0.0) && close(vel.z, 0.0));
    let push = step(&r, &input(v(8.0, 0.0, 0.0), Body::Anchor, Direction::Push)).unwrap();
    assert!(close(push.player_velocity.unwrap().x, -0.06));
    assert!(close(push.player_delta.x, -0.06));
}

// Catches a missing speed cap, and a cap that brakes a body already faster (a fall).
#[test]
fn player_speed_is_capped_but_never_braked() {
    let r = ForceRules::BURNING;
    let mut i = input(v(1.0, 0.0, 0.0), Body::Anchor, Direction::Pull);
    i.player_velocity = v(0.99, 0.0, 0.0);
    let vel = step(&r, &i).unwrap().player_velocity.unwrap();
    assert!(close(norm(vel), 1.0));
    i.player_velocity = v(0.0, -3.0, 0.0);
    let vel = step(&r, &i).unwrap().player_velocity.unwrap();
    assert!(close(norm(vel), 3.0), "a fall keeps its speed: {vel:?}");
    let mut f = input(v(1.0, 0.0, 0.0), Body::Anchor, Direction::Pull);
    f.player_velocity = v(1.39, 0.0, 0.0);
    let vel = step(&ForceRules::FLARED, &f)
        .unwrap()
        .player_velocity
        .unwrap();
    assert!(close(norm(vel), 1.4));
}

// Catches the coin jump failing: an on-ground item pushed downward must hold like an anchor.
#[test]
fn grounded_item_below_is_an_anchor_for_a_push() {
    let r = ForceRules::BURNING;
    let below = v(0.1, -1.0, 0.0);
    let jump = step(
        &r,
        &input(below, Body::Item { on_ground: true }, Direction::Push),
    )
    .unwrap();
    assert!(jump.anchored);
    let vel = jump.player_velocity.unwrap();
    // Stronger than gravity (0.08) at a metre: the player rises.
    assert!(vel.y > 0.08, "{vel:?}");
    // In the air, or pulled, or pushed sideways, the item moves instead.
    for (body, dir, target) in [
        (Body::Item { on_ground: false }, Direction::Push, below),
        (Body::Item { on_ground: true }, Direction::Pull, below),
        (
            Body::Item { on_ground: true },
            Direction::Push,
            v(2.0, -0.5, 0.0),
        ),
    ] {
        let out = step(&r, &input(target, body, dir)).unwrap();
        assert!(!out.anchored && out.player_velocity.is_none());
        assert!(out.target_velocity.is_some());
    }
}

// Catches a pushed item faster than its cap, and a pulled item that flies through the player.
#[test]
fn items_are_capped_and_settle_when_pulled_close() {
    let r = ForceRules::BURNING;
    let push = step(
        &r,
        &input(
            v(1.5, 0.0, 0.0),
            Body::Item { on_ground: false },
            Direction::Push,
        ),
    )
    .unwrap();
    let vel = push.target_velocity.unwrap();
    assert!(close(norm(vel), 2.0) && vel.x > 0.0);
    let pull = step(
        &r,
        &input(
            v(6.0, 0.0, 0.0),
            Body::Item { on_ground: true },
            Direction::Pull,
        ),
    )
    .unwrap();
    assert!(pull.target_velocity.unwrap().x < 0.0);
    let near = step(
        &r,
        &input(
            v(1.0, 0.0, 0.0),
            Body::Item { on_ground: false },
            Direction::Pull,
        ),
    )
    .unwrap();
    assert!(norm(near.target_velocity.unwrap()) <= 0.5 + 1e-9);
}

// Catches NaN reaching the engine and targets kept beyond the range.
#[test]
fn invalid_inputs_are_refused() {
    let r = ForceRules::BURNING;
    let mut i = input(v(f64::NAN, 0.0, 0.0), Body::Anchor, Direction::Pull);
    assert_eq!(step(&r, &i), Err(StepError::NonFinite));
    i.target = v(1.0, 0.0, 0.0);
    i.player_velocity = v(0.0, f64::INFINITY, 0.0);
    assert_eq!(step(&r, &i), Err(StepError::NonFinite));
    let far = input(v(16.0, 0.0, 0.0), Body::Anchor, Direction::Pull);
    assert_eq!(step(&r, &far), Err(StepError::OutOfRange));
    assert!(step(&ForceRules::FLARED, &far).is_ok());
    let same = input(v(0.0, 0.0, 0.0), Body::Anchor, Direction::Pull);
    assert_eq!(step(&r, &same), Err(StepError::Coincident));
}

#[test]
fn coin_damage_grows_with_speed() {
    assert_eq!(coin_damage(0.5), None);
    assert_eq!(coin_damage(f64::NAN), None);
    assert_eq!(coin_damage(1.0), Some(6.0));
    assert_eq!(coin_damage(2.0), Some(10.0));
}
