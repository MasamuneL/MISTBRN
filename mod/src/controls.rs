//! Key requests and the reserve HUD (P7a), server side.
//!
//! The client sends `mistbrn:control` (3 bytes, `allomancy_core::decode_control`) when the
//! player presses a mod key with nothing over the world. The sender is the connection's
//! player (`IPayloadContext::player`), never a value from the bytes. Requests wait in a small
//! inbox and are applied at the end of the server tick, each within the player's budget
//! (`RequestBudget`), resolved against the server's own state and run through the core's
//! `apply_command` with the input state the server can see. Malformed bytes are dropped and
//! logged; the session goes on.
//!
//! The server answers with `mistbrn:hud` (82 bytes) whenever what the HUD shows changed, at
//! most every `HUD_EVERY_TICKS`.
use crate::bridge::{Session, metal_name};
use crate::p4::{Powers, run_core_with};
use allomancy_core::{
    BurnState, Command, ControlError, ControlRequest, HudSnapshot, Pair, RequestBudget, Side, Vec3,
    decode_control, encode_hud, resolve_control,
};
use mod_api::{
    Host, IPayloadContext, IPayloadHandler, PayloadBytes, PayloadHandleError, PayloadId, Registrar,
    ServerHost,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, Mutex, PoisonError};

pub(crate) const CONTROL_CHANNEL: &str = "mistbrn:control";
pub(crate) const HUD_CHANNEL: &str = "mistbrn:hud";
/// Both channels' schema version, negotiated at join.
const CHANNEL_VERSION: &str = "1";
/// Requests waiting for the tick: at most this many from one player, the rest dropped.
const INBOX_PER_PLAYER: usize = 16;
const INBOX_MAX: usize = 256;
const HUD_EVERY_TICKS: u64 = 5;
/// One `rate` line per player per this many ticks.
const RATE_LOG_TICKS: u64 = 20;

type Decoded = Result<ControlRequest, ControlError>;

static INBOX: Mutex<VecDeque<(u128, Decoded)>> = Mutex::new(VecDeque::new());

struct ControlInbox;

impl IPayloadHandler for ControlInbox {
    fn handle(
        &self,
        payload: PayloadBytes,
        context: &mut dyn IPayloadContext,
    ) -> Result<(), PayloadHandleError> {
        // Not in play (no player): nothing to act for.
        let Ok(player) = context.player() else {
            return Ok(());
        };
        let uuid = player.uuid();
        let decoded = decode_control(&payload.0);
        let mut inbox = INBOX.lock().unwrap_or_else(PoisonError::into_inner);
        let mine = inbox.iter().filter(|(owner, _)| *owner == uuid).count();
        if mine < INBOX_PER_PLAYER && inbox.len() < INBOX_MAX {
            inbox.push_back((uuid, decoded));
        } else {
            eprintln!("MISTBRN event=control op=any result=rate");
        }
        Ok(())
    }
}

/// Declares both channels. The client's HUD handler is supplied by `client::register`.
pub(crate) fn register(registrar: &mut dyn Registrar, hud: Arc<dyn IPayloadHandler>) {
    let mut payloads = registrar.payload_handlers();
    let mut view = payloads.registrar(CHANNEL_VERSION);
    view.play_to_server(PayloadId(CONTROL_CHANNEL.into()), Arc::new(ControlInbox));
    view.play_to_client(PayloadId(HUD_CHANNEL.into()), Some(hud));
}

#[derive(Default)]
pub(crate) struct Controls {
    budgets: BTreeMap<u128, RequestBudget>,
    rate_logged: BTreeMap<u128, u64>,
    /// Players with a container menu open (`player_container_open` / `_close`). The player's
    /// own inventory and the chat are client screens the server never hears of.
    menus: BTreeSet<u128>,
    /// What each player's HUD last got, and when.
    sent: BTreeMap<u128, (HudSnapshot, u64)>,
}

fn op_name(request: ControlRequest) -> &'static str {
    match request {
        ControlRequest::CyclePair => "cycle",
        ControlRequest::Toggle(Side::First) => "toggle_first",
        ControlRequest::Toggle(Side::Second) => "toggle_second",
        ControlRequest::ToggleFlare => "flare",
        ControlRequest::Off => "off",
    }
}

pub(crate) fn pair_name(pair: Pair) -> &'static str {
    match pair {
        Pair::IronSteel => "iron_steel",
        Pair::TinPewter => "tin_pewter",
        Pair::ZincBrass => "zinc_brass",
        Pair::CopperBronze => "copper_bronze",
    }
}

fn command_name(command: &Command) -> String {
    match command {
        Command::Activate { metal } => format!("activate:{}", metal_name(*metal)),
        Command::Flare { metal } => format!("flare:{}", metal_name(*metal)),
        Command::Extinguish { metal: Some(metal) } => format!("off:{}", metal_name(*metal)),
        Command::Extinguish { metal: None } => "off:all".into(),
        Command::SelectPair { pair } => format!("pair:{}", pair_name(*pair)),
        _ => "other".into(),
    }
}

impl Controls {
    pub(crate) fn menu_opened(&mut self, uuid: u128) {
        self.menus.insert(uuid);
    }

    pub(crate) fn menu_closed(&mut self, uuid: u128) {
        self.menus.remove(&uuid);
    }

    /// Login, respawn, dimension change: send the HUD again on the next tick.
    pub(crate) fn resend(&mut self, uuid: u128) {
        self.sent.remove(&uuid);
    }

    pub(crate) fn logged_out(&mut self, uuid: u128) {
        self.budgets.remove(&uuid);
        self.rate_logged.remove(&uuid);
        self.menus.remove(&uuid);
        self.sent.remove(&uuid);
    }

    /// Applies the requests that arrived since the last tick, oldest first.
    pub(crate) fn apply(
        &mut self,
        host: &mut dyn ServerHost,
        session: &mut Session,
        powers: &mut Powers,
        tick: u64,
    ) {
        let pending: Vec<(u128, Decoded)> = INBOX
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .drain(..)
            .collect();
        for (uuid, decoded) in pending {
            let budget = self
                .budgets
                .entry(uuid)
                .or_insert_with(|| RequestBudget::new(tick));
            if !budget.allow(tick) {
                let last = self.rate_logged.get(&uuid).copied();
                if last.is_none_or(|at| tick.saturating_sub(at) >= RATE_LOG_TICKS) {
                    eprintln!("MISTBRN event=control op=any result=rate");
                    self.rate_logged.insert(uuid, tick);
                }
                continue;
            }
            let request = match decoded {
                Ok(request) => request,
                Err(error) => {
                    eprintln!(
                        "MISTBRN event=control op=invalid result=rejected:{}",
                        error.reason()
                    );
                    continue;
                }
            };
            self.apply_one(host, session, powers, uuid, request);
        }
    }

    fn apply_one(
        &mut self,
        host: &mut dyn ServerHost,
        session: &mut Session,
        powers: &mut Powers,
        uuid: u128,
        request: ControlRequest,
    ) {
        let op = op_name(request);
        let Some(player) = host.player(uuid) else {
            return;
        };
        let read: &dyn Host = host;
        let Some([x, y, z]) = player.entity().position(read) else {
            return;
        };
        // What the server can see of the player's input: no container menu open and not a
        // spectator. The client also sends nothing with a screen (chat, inventory) open.
        let allowed = !self.menus.contains(&uuid) && player.is_spectator(read) != Some(true);
        let Some(saved) = session.saved(uuid) else {
            eprintln!("MISTBRN event=control op={op} result=not_active");
            return;
        };
        let Some(command) = resolve_control(saved.state(), request) else {
            eprintln!("MISTBRN event=control op={op} result=nothing_burning changed=0");
            return;
        };
        let newly_lit = match &command {
            Command::Activate { metal } => {
                saved.state().metals[metal.index()].burn == BurnState::Off
            }
            _ => false,
        };
        let name = command_name(&command);
        let off = matches!(command, Command::Extinguish { .. });
        let position = Vec3 { x, y, z };
        let (result, changed) = run_core_with(session, uuid, position, command, allowed);
        if changed > 0 {
            if off {
                powers.clear(uuid, "off");
            } else if newly_lit {
                // A new lighting aims anew, as `/mistbrn-test burn` does.
                powers.clear(uuid, "relit");
            }
        }
        eprintln!(
            "MISTBRN event=control op={op} command={name} allowed={allowed} result={result} \
             changed={changed}"
        );
    }

    /// Sends each active player's HUD when it changed since the last send.
    pub(crate) fn sync_hud(&mut self, host: &mut dyn ServerHost, session: &Session, tick: u64) {
        let channel = PayloadId(HUD_CHANNEL.into());
        for player in host.players().server_players() {
            let uuid = player.uuid();
            let Some(saved) = session.saved(uuid) else {
                continue;
            };
            let hud = HudSnapshot::from_state(saved.state(), saved.rules());
            let due = match self.sent.get(&uuid) {
                None => true,
                Some((last, at)) => *last != hud && tick.saturating_sub(*at) >= HUD_EVERY_TICKS,
            };
            if !due {
                continue;
            }
            let bytes = PayloadBytes(encode_hud(&hud).to_vec());
            if let Err(error) = player.send_payload(host, &channel, bytes) {
                // Retried after the interval rather than every tick.
                eprintln!("MISTBRN event=hud_send_failed error={error:?}");
            }
            self.sent.insert(uuid, (hud, tick));
        }
    }
}
