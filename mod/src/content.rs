//! Connect item use and loot hooks to the P3 content rules.
use allomancy_core::{FILLED_VIALS, Metal, chest_sample, ore_sample};
use mod_api::{
    BlockDropsEvent, ClientHost, ContainerLootFilledEvent, GameMode, Hand, Host, ItemKey, ItemPart,
    ItemStack, LevelHost, LivingEntityUseItemFinishEvent, PlayerRef, Registrar, ServerHost,
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

struct Vial;

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
    ) -> Option<Drink> {
        let key = event.stack.item();
        let metal = FILLED_VIALS
            .iter()
            .find_map(|(id, metal)| (self.keys.get(id) == Some(&key)).then_some(*metal))?;
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
        Some(Drink {
            player,
            slot,
            stack: event.stack.clone(),
            metal,
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
        drop.stack = self.stack(sample)?;
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
