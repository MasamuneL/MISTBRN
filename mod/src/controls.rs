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
//! most every `HUD_EVERY_TICKS`, and with `mistbrn:progress` (the notebook, P5) whenever the
//! player's progress changed. The notebook's requests (buy a node, reveal a hint) are save
//! transactions (`Session::purchase`, `Session::reveal_hint`), not power commands.
use crate::bridge::{Session, metal_name};
use crate::p4::{Powers, run_core_with};
use allomancy_core::{
    BurnState, Command, ControlError, ControlRequest, HudSnapshot, Metal, PROGRESS_LEN, Pair,
    RequestBudget, Side, Vec3, decode_control, encode_hud, encode_progress_snapshot,
    resolve_control,
};
use mod_api::{
    Host, IPayloadContext, IPayloadHandler, PayloadBytes, PayloadHandleError, PayloadId, PlayerRef,
    Registrar, ServerHost, Text,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

pub(crate) const CONTROL_CHANNEL: &str = "mistbrn:control";
pub(crate) const HUD_CHANNEL: &str = "mistbrn:hud";
pub(crate) const PROGRESS_CHANNEL: &str = "mistbrn:progress";
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
/// Requests dropped by a full inbox since the last tick, logged once per tick outside the lock.
static DROPPED: AtomicU64 = AtomicU64::new(0);

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
        enqueue(player.uuid(), decode_control(&payload.0));
        Ok(())
    }
}

/// Test runs only (`/mistbrn-test flood`): queue `count` valid "all off" requests for
/// `uuid` through the same inbox limits a client's packets meet.
pub(crate) fn flood(uuid: u128, count: u32) {
    for _ in 0..count.min(1_000) {
        let bytes = allomancy_core::encode_control(ControlRequest::Off);
        enqueue(uuid, decode_control(&bytes));
    }
}

/// Test runs only (`/mistbrn-test inject <op>`): queue one request as the player's client
/// would send it, also while a screen is open (the client itself never sends then). K8 uses
/// it to check the server's own input check.
pub(crate) fn inject(uuid: u128, op: &str) {
    let (request, count) = match op {
        "cycle" => (ControlRequest::CyclePair, 1),
        "toggle_first" => (ControlRequest::Toggle(Side::First), 1),
        "toggle_second" => (ControlRequest::Toggle(Side::Second), 1),
        "flare" => (ControlRequest::ToggleFlare, 1),
        "off" => (ControlRequest::Off, 1),
        "select_tin_pewter" => (ControlRequest::SelectPair(Pair::TinPewter), 1),
        // P5: notebook requests as the client sends them (`buy:<code>:<rank>`,
        // `hint:<research>:<index>`), with an optional repeat count (`:<count>`, 1–4) queued
        // in this one call, so the copies reach the server in the same tick.
        other => match parse_notebook(other) {
            Some(parsed) => parsed,
            None => {
                eprintln!("MISTBRN event=test_command result=unknown");
                return;
            }
        },
    };
    let bytes = allomancy_core::encode_control(request);
    for _ in 0..count {
        enqueue(uuid, decode_control(&bytes));
    }
}

fn parse_notebook(op: &str) -> Option<(ControlRequest, u8)> {
    let mut parts = op.split(':');
    let kind = parts.next()?;
    let a: u8 = parts.next()?.parse().ok()?;
    let b: u8 = parts.next()?.parse().ok()?;
    let count: u8 = match parts.next() {
        None => 1,
        Some(count) => count.parse().ok().filter(|c| (1..=4).contains(c))?,
    };
    if parts.next().is_some() {
        return None;
    }
    let request = match kind {
        "buy" => ControlRequest::Purchase { code: a, rank: b },
        "hint" => ControlRequest::RevealHint {
            research: a,
            index: b,
        },
        _ => return None,
    };
    Some((request, count))
}

fn enqueue(uuid: u128, decoded: Decoded) {
    let mut inbox = INBOX.lock().unwrap_or_else(PoisonError::into_inner);
    let mine = inbox.iter().filter(|(owner, _)| *owner == uuid).count();
    if mine < INBOX_PER_PLAYER && inbox.len() < INBOX_MAX {
        inbox.push_back((uuid, decoded));
    } else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
}

/// Declares the channels. The client's handlers are supplied by `client`.
pub(crate) fn register(
    registrar: &mut dyn Registrar,
    hud: Arc<dyn IPayloadHandler>,
    progress: Arc<dyn IPayloadHandler>,
) {
    let mut payloads = registrar.payload_handlers();
    let mut view = payloads.registrar(CHANNEL_VERSION);
    view.play_to_server(PayloadId(CONTROL_CHANNEL.into()), Arc::new(ControlInbox));
    view.play_to_client(PayloadId(HUD_CHANNEL.into()), Some(hud));
    view.play_to_client(PayloadId(PROGRESS_CHANNEL.into()), Some(progress));
}

#[derive(Default)]
pub(crate) struct Controls {
    budgets: BTreeMap<u128, RequestBudget>,
    rate_logged: BTreeMap<u128, u64>,
    /// Players with a container menu open (`player_container_open` / `_close`). The player's
    /// own inventory and the chat are client screens the server never hears of.
    menus: BTreeSet<u128>,
    /// What each player's HUD last accepted (none until a send is queued) and the tick of
    /// the last attempt: a refused send is retried after the interval.
    sent: BTreeMap<u128, (Option<HudSnapshot>, u64)>,
    /// The progress snapshot each player's notebook last accepted (P5).
    sent_progress: BTreeMap<u128, (Option<[u8; PROGRESS_LEN]>, u64)>,
}

/// One line over the player's hotbar, with the metal's name when there is one.
fn hint(host: &mut dyn ServerHost, player: PlayerRef, key: &str, metal: Option<Metal>) {
    let args = metal
        .map(|metal| {
            vec![Text::translate(
                format!("hud.mistbrn.metal.{}", metal_name(metal)),
                Vec::new(),
            )]
        })
        .unwrap_or_default();
    let shown = player
        .action_bar(host, Text::translate(key, args))
        .unwrap_or(false);
    eprintln!("MISTBRN event=hint key={key} shown={shown}");
}

fn op_name(request: ControlRequest) -> &'static str {
    match request {
        ControlRequest::CyclePair => "cycle",
        ControlRequest::Toggle(Side::First) => "toggle_first",
        ControlRequest::Toggle(Side::Second) => "toggle_second",
        ControlRequest::ToggleFlare => "flare",
        ControlRequest::Off => "off",
        ControlRequest::SelectPair(_) => "select",
        ControlRequest::Purchase { .. } => "buy",
        ControlRequest::RevealHint { .. } => "hint",
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
        self.sent_progress.remove(&uuid);
    }

    /// Drops the player's waiting requests: one queued before a death, a dimension change or a
    /// logout must not act after it.
    pub(crate) fn purge(&mut self, uuid: u128) {
        INBOX
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|(owner, _)| *owner != uuid);
    }

    /// A stopped server: nothing waiting belongs to the next world.
    pub(crate) fn purge_all(&mut self) {
        INBOX.lock().unwrap_or_else(PoisonError::into_inner).clear();
        DROPPED.store(0, Ordering::Relaxed);
    }

    pub(crate) fn logged_out(&mut self, uuid: u128) {
        self.purge(uuid);
        self.budgets.remove(&uuid);
        self.rate_logged.remove(&uuid);
        self.menus.remove(&uuid);
        self.sent.remove(&uuid);
        self.sent_progress.remove(&uuid);
    }

    /// Applies the requests that arrived since the last tick, oldest first.
    pub(crate) fn apply(
        &mut self,
        host: &mut dyn ServerHost,
        session: &mut Session,
        powers: &mut Powers,
        tick: u64,
    ) {
        let dropped = DROPPED.swap(0, Ordering::Relaxed);
        if dropped > 0 {
            eprintln!("MISTBRN event=control op=any result=inbox_full dropped={dropped}");
        }
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
        // The notebook's requests: save transactions, refused with the input captured.
        match request {
            ControlRequest::Purchase { code, rank }
            | ControlRequest::RevealHint {
                research: code,
                index: rank,
            } => {
                if !allowed {
                    eprintln!("MISTBRN event=control op={op} result=InputCaptured");
                    return;
                }
                let result = if matches!(request, ControlRequest::Purchase { .. }) {
                    session.purchase(host, player, u16::from(code), rank)
                } else {
                    session.reveal_hint(host, player, u16::from(code), rank)
                };
                eprintln!(
                    "MISTBRN event=control op={op} result={}",
                    result.map_or_else(|error| error, |_| "ok".into())
                );
                return;
            }
            _ => {}
        }
        let Some(saved) = session.saved(uuid) else {
            eprintln!("MISTBRN event=control op={op} result=not_active");
            return;
        };
        let Some(command) = resolve_control(saved.state(), request) else {
            eprintln!("MISTBRN event=control op={op} result=nothing_burning changed=0");
            // No hint for a captured input (a container open, a spectator): nothing was pressed
            // in the game there.
            if allowed {
                hint(host, player, "message.mistbrn.nothing_burning", None);
            }
            return;
        };
        let newly_lit = match &command {
            Command::Activate { metal } => {
                saved.state().metals[metal.index()].burn == BurnState::Off
            }
            _ => false,
        };
        let name = command_name(&command);
        let lit_metal = match command {
            Command::Activate { metal } | Command::Flare { metal } => Some(metal),
            _ => None,
        };
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
        // A key that could not act says why, over the hotbar (none for a captured input: the
        // player is in a menu and pressed nothing there).
        if let Some(metal) = lit_metal {
            if result.starts_with("MetalLocked") {
                hint(host, player, "message.mistbrn.metal_locked", Some(metal));
            } else if result.starts_with("EmptyReserve") {
                hint(host, player, "message.mistbrn.metal_empty", Some(metal));
            }
        }
    }

    /// Sends each active player's progress snapshot (the notebook) when it changed. Changes
    /// are rare (an action, a purchase): no interval beyond a retry after a refused send.
    pub(crate) fn sync_progress(
        &mut self,
        host: &mut dyn ServerHost,
        session: &Session,
        tick: u64,
    ) {
        let channel = PayloadId(PROGRESS_CHANNEL.into());
        for player in host.players().server_players() {
            let uuid = player.uuid();
            let Some(saved) = session.saved(uuid) else {
                continue;
            };
            let bytes = encode_progress_snapshot(saved.progress());
            let due = match self.sent_progress.get(&uuid) {
                None => true,
                Some((last, at)) => {
                    *last != Some(bytes) && tick.saturating_sub(*at) >= HUD_EVERY_TICKS
                }
            };
            if !due {
                continue;
            }
            let accepted = match player.send_payload(host, &channel, PayloadBytes(bytes.to_vec())) {
                Ok(_) => Some(bytes),
                Err(error) => {
                    eprintln!("MISTBRN event=progress_send_failed error={error:?}");
                    self.sent_progress.get(&uuid).and_then(|(last, _)| *last)
                }
            };
            self.sent_progress.insert(uuid, (accepted, tick));
        }
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
                Some((last, at)) => {
                    *last != Some(hud) && tick.saturating_sub(*at) >= HUD_EVERY_TICKS
                }
            };
            if !due {
                continue;
            }
            let bytes = PayloadBytes(encode_hud(&hud).to_vec());
            let accepted = match player.send_payload(host, &channel, bytes) {
                Ok(_) => Some(hud),
                Err(error) => {
                    // Keep the last accepted snapshot: the next try comes after the interval
                    // even when nothing changes any more.
                    eprintln!("MISTBRN event=hud_send_failed error={error:?}");
                    self.sent.get(&uuid).and_then(|(last, _)| *last)
                }
            };
            self.sent.insert(uuid, (accepted, tick));
        }
    }
}
