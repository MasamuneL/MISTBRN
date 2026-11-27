//! Tin and pewter's powers (P8), client side: what only the player's own client can do.
//!
//! - Hearing's outline: the creatures the server lists (`mistbrn:senses`) glow in tin's colour,
//!   for this client only (`entity_outline`).
//! - Senses: the fog is farther while tin burns (`render_fog`), not with darkness or blindness.
//! - Pewter's jump: the local player's jump power (`living_jump_power`; the client moves it).
//! - The dodge's direction, from the movement keys (`movement_input`).
//!
//! All of it reads the HUD and the progress the server last sent (`client::lit`, `client::rules`).
use crate::client;
use allomancy_core::{Metal, decode_senses};
use mod_api::client::fog::{FogData, FogType};
use mod_api::{ClientHost, IPayloadContext, IPayloadHandler, PayloadBytes, PayloadHandleError};
use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

/// Tin's outline colour (0xRRGGBB).
const TIN_OUTLINE: u32 = 0x00C8_D2DC;

static SENSED: Mutex<Vec<u32>> = Mutex::new(Vec::new());
/// The local player's entity id (0: none yet), kept by the client tick.
static LOCAL: AtomicU32 = AtomicU32::new(0);
/// The last movement keys' direction for a dodge: 0 left, 1 right, 2 back, 3 forward.
static DIRECTION: AtomicU8 = AtomicU8::new(2);
/// What was last logged (test runs read these): the fog factor ×100 and the jump ×100.
static FOG_LOGGED: AtomicU32 = AtomicU32::new(100);
static JUMP_LOGGED: AtomicU32 = AtomicU32::new(100);

struct SensesInbox;

impl IPayloadHandler for SensesInbox {
    fn handle(
        &self,
        payload: PayloadBytes,
        _: &mut dyn IPayloadContext,
    ) -> Result<(), PayloadHandleError> {
        match decode_senses(&payload.0) {
            Some(ids) => {
                eprintln!("MISTBRN event=senses count={}", ids.len());
                *SENSED.lock().unwrap_or_else(PoisonError::into_inner) = ids;
            }
            None => eprintln!("MISTBRN event=senses_rejected len={}", payload.0.len()),
        }
        Ok(())
    }
}

pub(crate) fn handler() -> Arc<dyn IPayloadHandler> {
    Arc::new(SensesInbox)
}

/// The client tick: the local player's id, and nothing sensed once tin is out.
pub(crate) fn tick(host: &dyn ClientHost) {
    let id = host.local_player().map_or(0, |player| player.entity().id());
    LOCAL.store(id, Ordering::Relaxed);
    if client::lit(Metal::Tin).is_none() {
        SENSED
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
    }
}

pub(crate) fn logged_out() {
    SENSED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clear();
    LOCAL.store(0, Ordering::Relaxed);
}

pub(crate) fn outline(entity: u32, outline: &mut Option<u32>) {
    let Ok(sensed) = SENSED.try_lock() else {
        return;
    };
    if sensed.contains(&entity) {
        *outline = Some(TIN_OUTLINE);
    }
}

pub(crate) fn fog(host: &dyn ClientHost, fog_type: FogType, fog: &mut FogData) {
    if !matches!(fog_type, FogType::Atmospheric | FogType::Water) {
        return;
    }
    let factor = match client::lit(Metal::Tin) {
        Some(flared) if !client::blinded(host) => client::rules().fog(flared),
        _ => 1.0,
    };
    let shown = (factor * 100.0).round() as u32;
    if FOG_LOGGED.swap(shown, Ordering::Relaxed) != shown {
        eprintln!("MISTBRN event=fog factor={:.2}", factor);
    }
    if factor > 1.0 {
        fog.environmental_start *= factor;
        fog.environmental_end *= factor;
    }
}

/// The local player's jump power with pewter lit (only the client moves its player).
pub(crate) fn jump(entity: u32, power: &mut f32) {
    if entity == 0 || entity != LOCAL.load(Ordering::Relaxed) {
        return;
    }
    let factor = match client::lit(Metal::Pewter) {
        Some(_) => client::rules().jump(),
        None => 1.0,
    };
    let shown = (factor * 100.0).round() as u32;
    if JUMP_LOGGED.swap(shown, Ordering::Relaxed) != shown {
        eprintln!("MISTBRN event=jump factor={factor:.2}");
    }
    *power *= factor;
}

pub(crate) fn movement(left: bool, right: bool, back: bool, forward: bool) {
    let direction = match (left, right, back, forward) {
        (true, false, _, _) => 0,
        (false, true, _, _) => 1,
        (_, _, false, true) => 3,
        _ => 2,
    };
    DIRECTION.store(direction, Ordering::Relaxed);
}

pub(crate) fn direction() -> u8 {
    DIRECTION.load(Ordering::Relaxed)
}
