//! P3c alchemy: the alembic's batches, filling vials with water, beach salt and the empty
//! vial a furnace gives back after boiling a water vial into salt.
use allomancy_core::{
    ALEMBIC_INGREDIENT_SLOT, ALEMBIC_VIALS, BREW_TICKS, EMPTY_VIAL, SALT, WATER_VIAL, beach_salt,
    brew_result, is_alembic_ingredient, is_alembic_vial,
};
use mod_api::{
    BlockDropsEvent, BlockEntityPart, BlockEntityTickEvent, BlockEntityTickReply,
    BlockEntityTicksEvent, BlockPart, ClientHost, ClipBlock, ClipFluid, FurnaceSmeltEvent,
    GameMode, Hand, Host, HostTransaction, Identifier, InteractionResult, ItemClick, ItemStack,
    LevelHost, LevelRef, MenuContainer, MenuPart, MenuSlotRulesEvent, ModContainer, Registrar,
    Screen, ScreenDrawEvent, ScreenEvent, ScreenTooltipEvent, ServerHost, Tag, UseWithoutItemEvent,
};

pub(crate) const ALEMBIC: &str = "mistbrn:alembic";
const OFFHAND_SLOT: u32 = 40;
/// The saved batch: ticks brewed so far and the ingredient they were brewed with.
const PROGRESS: &str = "progress";
const BREWING: &str = "brewing";
/// How far the player reaches water with a vial (vanilla's block interaction range).
const REACH: f64 = 4.5;
/// Screen layout (GUI pixels from its top left), as vanilla's brewing stand: the ingredient
/// on top, the three vials below and the batch bar between them.
const VIAL_SLOTS: [[f32; 2]; ALEMBIC_VIALS] = [[56.0, 51.0], [79.0, 58.0], [102.0, 51.0]];
const INGREDIENT_AT: [f32; 2] = [79.0, 17.0];
const BAR: [f32; 4] = [99.0, 17.0, 6.0, 30.0];
const BAR_TRACK: u32 = 0xFF37_3A40;
const BAR_EDGE: u32 = 0xFF1A_1C20;
const BAR_FILL: u32 = 0xFFB5_9654;

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
impl BlockPart for Alembic {
    /// An empty-hand click opens the alembic's menu (vanilla's container blocks do the same).
    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        let player = host.player(click.player)?;
        let level = player.level(&*host)?;
        let entity = host.block_entity(level, click.pos)?;
        let opened = player.open_menu(host, entity).unwrap_or(false);
        opened.then_some(InteractionResult::SuccessServer)
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for Alembic {
    fn block_entity_ticks(&self, _: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    fn screen(&self, _: &dyn Host, event: &mut ScreenEvent<'_>) -> Option<Screen> {
        (event.block == ALEMBIC).then(|| {
            let mut slots: Vec<Option<[f32; 2]>> = VIAL_SLOTS.iter().copied().map(Some).collect();
            slots.push(Some(INGREDIENT_AT));
            Screen {
                width: 176.0,
                height: 166.0,
                slots,
                inventory_y: 84.0,
                inventory_x: 8.0,
                ..Screen::default()
            }
        })
    }

    /// The batch bar fills from the top as the batch brews (`data[0]`, its ticks).
    fn screen_draw(&self, _: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
        if event.block != ALEMBIC {
            return;
        }
        let [ox, oy] = event.origin;
        let [x, y, w, h] = BAR;
        let draw = &mut *event.draw;
        draw.fill([ox + x - 1.0, oy + y - 1.0, w + 2.0, h + 2.0], BAR_EDGE);
        draw.fill([ox + x, oy + y, w, h], BAR_TRACK);
        let share = bar_share(event.data.first().copied().unwrap_or(0));
        if share > 0.0 {
            draw.fill([ox + x, oy + y, w, (h * share).max(1.0)], BAR_FILL);
        }
    }

    /// Over the bar: how long the batch has left, in seconds.
    fn screen_tooltip(&self, host: &dyn ClientHost, event: &mut ScreenTooltipEvent<'_>) {
        let [x, y, w, h] = BAR;
        let [ax, ay] = event.at;
        if event.block != ALEMBIC || ax < x || ax >= x + w || ay < y || ay >= y + h {
            return;
        }
        let ticks = u32::try_from(event.data.first().copied().unwrap_or(0)).unwrap_or(0);
        let line = if ticks == 0 {
            crate::hud::tr(host, "container.mistbrn.alembic.idle")
        } else {
            let left = BREW_TICKS.saturating_sub(ticks).div_ceil(20);
            crate::hud::tr_args(host, "container.mistbrn.alembic.left", &[left.to_string()])
        };
        event.lines.push(line);
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
        // A different ingredient, or a different set of vial slots, starts a new batch: a vial
        // put in late cannot ride on ticks it did not brew.
        let slots: String = targets.iter().map(|(slot, _)| slot.to_string()).collect();
        let brewing = Tag::String(format!("{ingredient_id}@{slots}").as_str().into());
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

/// How full the batch bar is, 0 to 1, from the batch's ticks.
fn bar_share(ticks: i32) -> f32 {
    let total = BREW_TICKS as f32;
    (ticks.max(0) as f32).min(total) / total
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

    /// Takes the sand broken in `dimension`; the caller resolves it with no lock held.
    pub(crate) fn take(&mut self, dimension: &str) -> Vec<[i32; 3]> {
        if self.pending.is_empty() {
            return Vec::new();
        }
        let (here, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.pending)
            .into_iter()
            .partition(|(dim, _)| dim == dimension);
        self.pending = rest;
        here.into_iter().map(|(_, pos)| pos).collect()
    }
}

/// Drops salt for the beach sand taken from [`BeachSalt`]. Runs outside its mutex: dropping an
/// item can run other mods' hooks, which may break blocks and re-enter `block_drops`.
pub(crate) fn drop_beach_salt(
    host: &mut dyn ServerHost,
    level: LevelRef,
    dimension: &str,
    due: Vec<[i32; 3]>,
) {
    for pos in due {
        let read: &dyn Host = host;
        let beach = level
            .biome_at(read, pos)
            .and_then(|biome| level.biome_in_tag(read, &biome, "minecraft:is_beach"));
        let time = level.game_time(read).and_then(|t| u64::try_from(t).ok());
        let (Some(true), Some(time)) = (beach, time) else {
            continue;
        };
        if !beach_salt(dimension, pos, time) {
            continue;
        }
        if let Some(salt) = stack_of(read, SALT, 1) {
            let dropped = matches!(level.pop_resource(host, pos, salt), Ok(Some(_)));
            eprintln!("MISTBRN event=beach_salt pos={pos:?} dropped={dropped}");
        }
    }
}

/// A furnace that boiled a water vial into salt gives the empty vial back, above itself.
pub(crate) fn furnace_smelt(host: &mut dyn ServerHost, event: &FurnaceSmeltEvent<'_>) {
    if event.item != WATER_VIAL || event.result != SALT {
        return;
    }
    let read: &dyn Host = host;
    // The hook names no level: take the one that has the furnace at `pos`, the level in place first.
    let level = [
        host.dimension(),
        "minecraft:overworld",
        "minecraft:the_nether",
        "minecraft:the_end",
    ]
    .into_iter()
    .filter_map(|dimension| read.level(dimension))
    .find(|level| {
        level
            .block(read, event.pos)
            .is_some_and(|block| block.id(read) == "minecraft:furnace")
    });
    let (Some(level), Some(vial)) = (level, stack_of(read, EMPTY_VIAL, 1)) else {
        eprintln!("MISTBRN event=furnace_vial_lost pos={:?}", event.pos);
        return;
    };
    let [x, y, z] = event.pos;
    // spawn_item, not pop_resource: the vial is the furnace's input coming back, not a block
    // drop, so it must not depend on the doTileDrops rule.
    let (x, y, z) = (f64::from(x) + 0.5, f64::from(y) + 1.1, f64::from(z) + 0.5);
    let dropped = matches!(level.spawn_item(host, x, y, z, vial, 10), Ok(Some(_)));
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

    #[test]
    fn the_bar_tracks_the_batch_and_stays_in_range() {
        assert_eq!(bar_share(0), 0.0);
        assert_eq!(bar_share(300), 0.5);
        assert_eq!(bar_share(-5), 0.0);
        assert_eq!(bar_share(10_000), 1.0);
    }
}
