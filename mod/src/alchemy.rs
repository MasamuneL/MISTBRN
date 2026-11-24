//! P3c alchemy: the alembic's batches, filling vials with water, beach salt and the empty
//! vial a furnace gives back after boiling a water vial into salt.
use allomancy_core::{
    ALEMBIC_INGREDIENT_SLOT, ALEMBIC_VIALS, BREW_TICKS, EMPTY_VIAL, SALT, WATER_VIAL, beach_salt,
    brew_result, is_alembic_ingredient, is_alembic_vial,
};
use mod_api::{
    BlockDropsEvent, BlockEntityPart, BlockEntityTickEvent, BlockEntityTickReply,
    BlockEntityTicksEvent, BlockPart, ClipBlock, ClipFluid, FurnaceSmeltEvent, GameMode, Hand,
    Host, HostTransaction, Identifier, ItemClick, ItemStack, LevelHost, LevelRef, MenuContainer,
    MenuPart, MenuSlotRulesEvent, ModContainer, Registrar, ServerHost, Tag,
};

pub(crate) const ALEMBIC: &str = "mistbrn:alembic";
const OFFHAND_SLOT: u32 = 40;
/// The saved batch: ticks brewed so far and the ingredient they were brewed with.
const PROGRESS: &str = "progress";
const BREWING: &str = "brewing";
/// How far the player reaches water with a vial (vanilla's block interaction range).
const REACH: f64 = 4.5;

fn stack_of(host: &dyn Host, id: &str, count: u32) -> Option<ItemStack> {
    let key = host.item_key(&Identifier::parse(id).ok()?)?;
    ItemStack::new(key, count)
}

pub(crate) fn register(r: &mut dyn Registrar) {
    let block = r.block_key(ALEMBIC);
    let kind = r.block_entity_type("alembic").blocks(&["alembic"]).build();
    if let Err(error) = r.block_entity_part(&kind, Alembic) {
        eprintln!("MISTBRN event=alembic_register_failed reason={error:?}");
    }
    // Only the menu reaches the slots: no hopper face, so batches stay hand-fed.
    r.block_container(
        &block,
        ModContainer {
            slots: ALEMBIC_VIALS + 1,
            faces: Default::default(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            menu: Some(ALEMBIC.into()),
        },
    );
    r.menu_part(ALEMBIC, AlembicMenu);
}

struct AlembicMenu;

#[mod_api::horizon_part]
impl MenuPart for AlembicMenu {
    /// The server checks every placement: vial slots take one brewable or brewed vial each,
    /// the ingredient slot takes salt or flakes.
    fn menu_slot_rules(&self, host: &dyn Host, event: &mut MenuSlotRulesEvent<'_>) {
        if event.container != MenuContainer::Main {
            return;
        }
        let id = event.stack.item().id(host);
        if event.slot < ALEMBIC_VIALS {
            event.may_place &= is_alembic_vial(&id);
            event.max_stack = 1;
        } else if event.slot == ALEMBIC_INGREDIENT_SLOT {
            event.may_place &= is_alembic_ingredient(&id);
        }
    }
}

struct Alembic;

#[mod_api::horizon_part]
impl BlockPart for Alembic {}

#[mod_api::horizon_part]
impl BlockEntityPart for Alembic {
    fn block_entity_ticks(&self, _: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One tick of a batch. The whole entity (slots, saved progress, menu data) is written
    /// back at once when the tick ends, so a finished batch consumes its ingredient and
    /// converts its vials together or not at all.
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let read: &dyn Host = host;
        let entity = &mut event.entity;
        let ingredient = entity.items.get(ALEMBIC_INGREDIENT_SLOT).cloned().flatten();
        let ingredient_id = ingredient.as_ref().map(|stack| stack.item().id(read));
        let targets: Vec<(usize, &'static str)> = (0..ALEMBIC_VIALS)
            .filter_map(|slot| {
                let vial = entity.items.get(slot)?.as_ref()?;
                let result = brew_result(ingredient_id.as_deref()?, &vial.item().id(read))?;
                Some((slot, result))
            })
            .collect();
        let saved = match entity.nbt.get(&PROGRESS.into()) {
            Some(Tag::Int(ticks)) => u32::try_from(*ticks).unwrap_or(0),
            _ => 0,
        };
        let (Some(ingredient), Some(ingredient_id)) = (ingredient, ingredient_id) else {
            return idle(event, saved);
        };
        if targets.is_empty() {
            return idle(event, saved);
        }
        // A different ingredient starts a new batch.
        let brewing = Tag::String(ingredient_id.as_str().into());
        let (mut progress, done) =
            advance(saved, entity.nbt.get(&BREWING.into()) == Some(&brewing));
        if done {
            let mut converted = Vec::new();
            for &(slot, result) in &targets {
                let count = entity.items[slot].as_ref().map_or(1, ItemStack::count);
                converted.push((slot, stack_of(read, result, count)));
            }
            if converted.iter().any(|(_, stack)| stack.is_none()) {
                // A result item the game does not know: leave the batch untouched.
                *event.reply = BlockEntityTickReply::Unchanged;
                return;
            }
            for (slot, stack) in converted {
                entity.items[slot] = stack;
            }
            entity.items[ALEMBIC_INGREDIENT_SLOT] = ingredient.shrink(1);
            eprintln!(
                "MISTBRN event=alembic_batch pos={:?} ingredient={ingredient_id} vials={}",
                event.pos,
                targets.len()
            );
            progress = 0;
        }
        let ticks = i32::try_from(progress).unwrap_or(0);
        entity.nbt.insert(PROGRESS.into(), Tag::Int(ticks));
        entity.nbt.insert(BREWING.into(), brewing);
        entity.data = vec![ticks];
    }
}

/// The batch's next tick count and whether this tick finishes it. A different ingredient
/// than the one the saved ticks were brewed with starts a new batch.
fn advance(saved: u32, same_ingredient: bool) -> (u32, bool) {
    let progress = if same_ingredient { saved + 1 } else { 1 };
    (progress, progress >= BREW_TICKS)
}

/// Nothing to brew: a running batch resets; an idle alembic writes nothing.
fn idle(event: &mut BlockEntityTickEvent<'_>, saved: u32) {
    let entity = &mut event.entity;
    if saved == 0 && entity.data.first() == Some(&0) {
        *event.reply = BlockEntityTickReply::Unchanged;
        return;
    }
    entity.nbt.remove(&PROGRESS.into());
    entity.nbt.remove(&BREWING.into());
    entity.data = vec![0];
}

/// Fills one empty vial from the water the player looks at (server side), as a glass bottle
/// does; the water stays. Returns whether it filled one.
pub(crate) fn fill_vial(host: &mut dyn ServerHost, click: &ItemClick<'_>) -> bool {
    let read: &dyn Host = host;
    if click.stack.item().id(read) != EMPTY_VIAL {
        return false;
    }
    let Some(player) = host.player(click.player) else {
        return false;
    };
    if matches!(player.game_mode(read), Some(GameMode::Spectator) | None) {
        return false;
    }
    let Some(level) = player.level(read) else {
        return false;
    };
    if looked_at_water(read, level, player.entity()).is_none() {
        return false;
    }
    let slot = match click.hand {
        Hand::Main => player
            .selected_slot(read)
            .and_then(|slot| u32::try_from(slot).ok()),
        Hand::Off => Some(OFFHAND_SLOT),
    };
    let (Some(slot), Some(water)) = (slot, stack_of(read, WATER_VIAL, 1)) else {
        return false;
    };
    let expected = click.stack.clone();
    let result = host.transaction(|host, _tx| {
        let current = crate::bridge::slot_stack(host, player, slot);
        if current.as_ref() != Some(&expected) {
            return Err("item_not_in_slot".to_string());
        }
        let remaining = expected.clone().shrink(1);
        let replacement = remaining.clone().or_else(|| Some(water.clone()));
        player
            .set_inventory_slot(host, slot, replacement)
            .map_err(|e| format!("set_slot:{e:?}"))?;
        if remaining.is_some() {
            player
                .give_stack(host, water.clone())
                .map_err(|e| format!("give:{e:?}"))?;
        }
        Ok(())
    });
    match result {
        Ok(()) => {
            eprintln!("MISTBRN event=vial_filled slot={slot}");
            true
        }
        Err(error) => {
            eprintln!("MISTBRN event=vial_fill_refused reason={error}");
            false
        }
    }
}

/// The first water source along the view, before the first block the ray hits. `clip`
/// ignores fluids at this pin (`ClipFluid::None` only), so the ray is walked in steps.
fn looked_at_water(host: &dyn Host, level: LevelRef, body: mod_api::EntityRef) -> Option<[i32; 3]> {
    let eye = body.eye_position(host)?;
    let look = body.view_vector(host)?;
    let end = [0, 1, 2].map(|i| eye[i] + look[i] * REACH);
    let reach = level
        .clip(host, eye, end, ClipBlock::Outline, ClipFluid::None)
        .filter(|hit| !hit.miss)
        .map_or(REACH, |hit| {
            let d = [0, 1, 2].map(|i| hit.location[i] - eye[i]);
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        });
    let water = host.fluid_key("minecraft:water")?;
    let steps = (reach / 0.1).ceil() as u32;
    (0..=steps).find_map(|step| {
        let t = (f64::from(step) * 0.1).min(reach);
        let pos = [0, 1, 2].map(|i| (eye[i] + look[i] * t).floor() as i32);
        let fluid = level.fluid_state(host, pos)?;
        (fluid.fluid == water && fluid.source).then_some(pos)
    })
}

/// Sand broken in a beach biome, waiting for its level's tick: `block_drops` only reads
/// blocks, so the biome and the game time are looked up there.
#[derive(Default)]
pub(crate) struct BeachSalt {
    pending: Vec<(String, [i32; 3])>,
}

impl BeachSalt {
    pub(crate) fn block_drops(&mut self, level: &dyn LevelHost, event: &BlockDropsEvent<'_>) {
        if event.breaker.is_none() || event.cancelled {
            return;
        }
        if level.state_block(event.state) != Some("minecraft:sand") {
            return;
        }
        // Bounded: a level that never ticks again cannot grow the queue without end.
        if self.pending.len() < 256 {
            let pos = [event.pos.x, event.pos.y, event.pos.z];
            self.pending
                .push((event.dimension.key().id().to_string(), pos));
        }
    }

    pub(crate) fn tick_level(&mut self, host: &mut dyn ServerHost, level: LevelRef) {
        let Some(dimension) = level.dimension().id(host) else {
            return;
        };
        let (here, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.pending)
            .into_iter()
            .partition(|(dim, _)| *dim == dimension);
        self.pending = rest;
        for (_, pos) in here {
            let read: &dyn Host = host;
            let beach = level
                .biome_at(read, pos)
                .and_then(|biome| level.biome_in_tag(read, &biome, "minecraft:is_beach"));
            let time = level.game_time(read).and_then(|t| u64::try_from(t).ok());
            let (Some(true), Some(time)) = (beach, time) else {
                continue;
            };
            if !beach_salt(&dimension, pos, time) {
                continue;
            }
            if let Some(salt) = stack_of(read, SALT, 1) {
                let dropped = level.pop_resource(host, pos, salt).is_ok();
                eprintln!("MISTBRN event=beach_salt pos={pos:?} dropped={dropped}");
            }
        }
    }
}

/// A furnace that boiled a water vial into salt gives the empty vial back, above itself.
pub(crate) fn furnace_smelt(host: &mut dyn ServerHost, event: &FurnaceSmeltEvent<'_>) {
    if event.item != WATER_VIAL {
        return;
    }
    let read: &dyn Host = host;
    let level = host.level(host.dimension());
    let (Some(level), Some(vial)) = (level, stack_of(read, EMPTY_VIAL, 1)) else {
        eprintln!("MISTBRN event=furnace_vial_lost pos={:?}", event.pos);
        return;
    };
    let [x, y, z] = event.pos;
    let dropped = level.pop_resource(host, [x, y + 1, z], vial).is_ok();
    eprintln!(
        "MISTBRN event=furnace_vial pos={:?} dropped={dropped}",
        event.pos
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_batch_takes_brew_ticks_and_restarts_with_another_ingredient() {
        let mut progress = 0;
        let mut ticks = 0;
        loop {
            let (next, done) = advance(progress, true);
            ticks += 1;
            if done {
                break;
            }
            progress = next;
        }
        assert_eq!(ticks, BREW_TICKS);
        assert_eq!(advance(599, false), (1, false));
        assert_eq!(advance(0, false), (1, false));
    }
}
