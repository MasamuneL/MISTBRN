//! Connect item use and loot hooks to the P3 content rules.
use allomancy_core::{
    FILLED_VIALS, Metal, Pair, UNLOCK_ITEMS, chest_sample, ore_sample, sample_count,
};
use mod_api::{
    BlockDropsEvent, ClientHost, ContainerLootFilledEvent, GameMode, Hand, Host, ItemKey, ItemPart,
    ItemStack, LevelHost, LivingEntityUseItemFinishEvent, PlayerRef, Registrar, ServerHost, Tag,
    UseAnimation, UseAnimationEvent, UseDurationEvent,
};
use std::collections::BTreeMap;

const OFFHAND_SLOT: u32 = 40;

#[derive(Default)]
pub(crate) struct Content {
    keys: BTreeMap<&'static str, ItemKey>,
}

pub(crate) struct Drink {
    pub player: PlayerRef,
    pub slot: u32,
    pub stack: ItemStack,
    pub metal: Metal,
}

/// One unlock item whose use finished, waiting for its transaction.
pub(crate) struct Study {
    pub player: PlayerRef,
    pub slot: u32,
    pub stack: ItemStack,
    pub pair: Pair,
}

/// A finished use of one of our items, applied after the finish hook wrote the hand back.
pub(crate) enum Finished {
    Drink(Drink),
    Study(Study),
}

impl Finished {
    pub(crate) fn player(&self) -> PlayerRef {
        match self {
            Self::Drink(drink) => drink.player,
            Self::Study(study) => study.player,
        }
    }

    pub(crate) fn slot(&self) -> u32 {
        match self {
            Self::Drink(drink) => drink.slot,
            Self::Study(study) => study.slot,
        }
    }
}

struct Vial;

struct Primer;

#[mod_api::horizon_part]
impl ItemPart for Primer {
    fn use_duration(&self, _: &dyn Host, event: &mut UseDurationEvent<'_>) {
        event.duration = 32;
    }

    fn use_animation(&self, _: &dyn ClientHost, event: &mut UseAnimationEvent<'_>) {
        *event.animation = UseAnimation::None;
    }
}

#[mod_api::horizon_part]
impl ItemPart for Vial {
    fn use_duration(&self, _: &dyn Host, event: &mut UseDurationEvent<'_>) {
        event.duration = 32;
    }

    fn use_animation(&self, _: &dyn ClientHost, event: &mut UseAnimationEvent<'_>) {
        *event.animation = UseAnimation::Drink;
    }
}

impl Content {
    pub(crate) fn register(&mut self, r: &mut dyn Registrar) {
        for (id, _) in FILLED_VIALS {
            let key = r.item_key(id);
            r.item_part(key, Vial);
            self.keys.insert(id, key);
        }
        for (id, _) in UNLOCK_ITEMS {
            let key = r.item_key(id);
            r.item_part(key, Primer);
            self.keys.insert(id, key);
        }
        for id in [
            "mistbrn:raw_tin",
            "mistbrn:raw_zinc",
            "minecraft:raw_copper",
            "minecraft:raw_iron",
        ] {
            self.keys.insert(id, r.item_key(id));
        }
    }

    fn stack(&self, id: &str) -> Option<ItemStack> {
        ItemStack::new(*self.keys.get(id)?, 1)
    }

    pub(crate) fn finish(
        &self,
        host: &dyn ServerHost,
        event: &LivingEntityUseItemFinishEvent<'_>,
    ) -> Option<Finished> {
        let key = event.stack.item();
        let is = |id: &str| self.keys.get(id) == Some(&key);
        let item = FILLED_VIALS
            .iter()
            .find_map(|&(id, metal)| is(id).then_some(Ok(metal)))
            .or_else(|| {
                UNLOCK_ITEMS
                    .iter()
                    .find_map(|&(id, pair)| is(id).then_some(Err(pair)))
            })?;
        // This event gives an entity ID for a mob. A player is exposed
        // through the server's acting-player context during the callback.
        if event.entity.is_some() {
            return None;
        }
        let player = host.players().acting_player()?;
        if !matches!(
            player.game_mode(host)?,
            GameMode::Survival | GameMode::Adventure
        ) {
            return None;
        }
        let slot = match event.hand {
            Hand::Main => u32::try_from(player.selected_slot(host)?).ok()?,
            Hand::Off => OFFHAND_SLOT,
        };
        let stack = event.stack.clone();
        Some(match item {
            Ok(metal) => Finished::Drink(Drink {
                player,
                slot,
                stack,
                metal,
            }),
            Err(pair) => Finished::Study(Study {
                player,
                slot,
                stack,
                pair,
            }),
        })
    }

    pub(crate) fn block_drops(
        &self,
        level: &dyn LevelHost,
        event: &mut BlockDropsEvent<'_>,
    ) -> Option<()> {
        if event.breaker.is_none() || event.tool.is_none() {
            return None;
        }
        let ore = level.state_block(event.state)?.strip_prefix("minecraft:")?;
        let pos = [event.pos.x, event.pos.y, event.pos.z];
        let (vanilla, sample) = ore_sample(ore, event.dimension.key().id(), pos)?;
        let vanilla = self.keys.get(vanilla)?;
        // Copy the vanilla raw drop's motion and position. Silk Touch and tools that cannot
        // harvest the ore produce no eligible stack.
        let mut drop = event
            .drops
            .iter()
            .find(|drop| drop.stack.item() == *vanilla)?
            .clone();
        let fortune = event.tool.map_or(0, fortune_level);
        let count = sample_count(fortune, event.dimension.key().id(), pos);
        drop.stack = ItemStack::new(*self.keys.get(sample)?, u32::from(count))?;
        event.drops.push(drop);
        Some(())
    }

    pub(crate) fn chest_drops(&self, event: &mut ContainerLootFilledEvent<'_>) -> Option<()> {
        let context = event.context;
        context.opener?;
        let pos = context.block.or_else(|| {
            context
                .entity
                .map(|_| context.origin.map(|coord| coord.floor() as i32))
        })?;
        event
            .drops
            .push(self.stack(chest_sample(context.table, context.dimension, pos)?)?);
        Some(())
    }
}

/// The tool's Fortune level from the `minecraft:enchantments` component in its patch (26.3 form:
/// a map from enchantment id to level), 0 when absent. `block_drops` only gets a `LevelHost`,
/// which cannot resolve an item's default components (`ItemResource::value` needs a `Host`):
/// Fortune that a data pack or mod puts in a tool's defaults is not seen. Vanilla tools have
/// none; enchanting tables, anvils, `/enchant` and loot all write the patch.
fn fortune_level(tool: &ItemStack) -> u8 {
    let Some(Tag::Compound(levels)) = tool
        .components()
        .and_then(|components| components.as_nbt().get(&"minecraft:enchantments".into()))
    else {
        return 0;
    };
    match levels.get(&"minecraft:fortune".into()) {
        Some(Tag::Int(level)) => u8::try_from((*level).clamp(0, 255)).unwrap_or(0),
        _ => 0,
    }
}
