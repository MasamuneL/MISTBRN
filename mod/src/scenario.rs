//! In-game test scenarios, run only when `MISTBRN_TEST_SCENARIO` names one.
//!
//! Without the variable the runner is inert: no reads, writes or log lines. A scenario runs once
//! per process, on the first server tick where its precondition holds, and reports
//! `MISTBRN_TEST scenario=<name> key=value ...` for `tools/native-test.sh`.
//!
//! The P2 regression scenarios drive the same vial transaction as normal item use.
use crate::bridge::{Session, Slot, metal_name};
use crate::content::Drink;
use crate::save::{self, Blocked};
use allomancy_core::{
    BurnState, EMPTY_VIAL, Metal, SaveError, SavedPlayer, UNLOCK_ITEMS, VIAL_MILLI, encode_save,
    update_player,
};
use mod_api::{Compound, Host, HostTransaction, Identifier, ItemStack, PlayerRef, ServerHost, Tag};

const PERSISTED: &str = "PlayerPersisted";
const PROBE_KEY: &str = "mistbrn_probe";
const ABORT_KEY: &str = "mistbrn_abort_probe";
const PROBE_BYTES: [u8; 3] = [1, 2, 3];
const IRON_VIAL: &str = "mistbrn:vial_iron";
const SMALL_MILLI: i64 = 1_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scenario {
    /// G5: write a probe into `NeoForgeData.PlayerPersisted`, read it back after a restart.
    PdRoundtrip,
    /// G6: a transaction that writes inventory and persistent data, then aborts.
    TxAbort,
    /// G7: does the player's raw entity NBT expose `NeoForgeData`?
    RawRoot,
    /// Report the loaded save and inventory, change nothing.
    Report,
    /// Unlock iron, then drink one vial from slot 0 (25 000 milli).
    Drink,
    /// As Drink, with a failure injected inside the transaction after both writes.
    DrinkFail,
    /// Drink, then burn iron.
    Burn,
    /// Unlock iron, credit 1 000 milli for one vial, then burn it to exhaustion.
    BurnSmall,
    /// Overwrite the save with its checksum broken, then stop writing for this session.
    CorruptCrc,
    /// Overwrite the save labelled with the schema after the current one, then stop writing.
    FutureSchema,
}

impl Scenario {
    const ALL: [Self; 10] = [
        Self::PdRoundtrip,
        Self::TxAbort,
        Self::RawRoot,
        Self::Report,
        Self::Drink,
        Self::DrinkFail,
        Self::Burn,
        Self::BurnSmall,
        Self::CorruptCrc,
        Self::FutureSchema,
    ];
    fn name(self) -> &'static str {
        match self {
            Self::PdRoundtrip => "pd-roundtrip",
            Self::TxAbort => "tx-abort",
            Self::RawRoot => "raw-root",
            Self::Report => "report",
            Self::Drink => "drink",
            Self::DrinkFail => "drink-fail",
            Self::Burn => "burn",
            Self::BurnSmall => "burn-small",
            Self::CorruptCrc => "corrupt-crc",
            Self::FutureSchema => "future-schema",
        }
    }
    fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.name() == name)
    }
    /// Scenarios that drive the real session need it loaded; the P0b probes do not.
    fn needs_session(self) -> bool {
        !matches!(self, Self::PdRoundtrip | Self::TxAbort | Self::RawRoot)
    }
    fn needs_vial(self) -> bool {
        matches!(
            self,
            Self::Drink | Self::DrinkFail | Self::Burn | Self::BurnSmall
        )
    }
}

pub(crate) struct Runner {
    pending: Option<Scenario>,
    /// `MISTBRN_TEST_FAIL_VIAL=1`: fail every vial and unlock-item use inside its transaction.
    fail_use: bool,
}

/// P4 scenarios drive everything through `/mistbrn-test`: the runner has nothing to do.
const COMMANDS_ONLY: &str = "p4";

/// Whether the `/mistbrn-test` commands exist: only with `MISTBRN_TEST_SCENARIO=p4` exactly.
/// Their credit writes the save without a vial, so no other value (a typo, another scenario)
/// turns them on.
pub(crate) fn commands_enabled() -> bool {
    std::env::var("MISTBRN_TEST_SCENARIO").as_deref() == Ok(COMMANDS_ONLY)
}

impl Runner {
    pub(crate) fn from_env() -> Self {
        let pending = std::env::var("MISTBRN_TEST_SCENARIO")
            .ok()
            .filter(|name| !name.is_empty() && name != COMMANDS_ONLY)
            .and_then(|name| {
                let parsed = Scenario::parse(&name);
                if parsed.is_none() {
                    eprintln!("MISTBRN_TEST scenario={name} error=unknown_scenario");
                }
                parsed
            });
        Self {
            pending,
            fail_use: std::env::var("MISTBRN_TEST_FAIL_VIAL").as_deref() == Ok("1"),
        }
    }

    pub(crate) fn fail_use(&self) -> bool {
        self.fail_use
    }

    pub(crate) fn tick(&mut self, host: &mut dyn ServerHost, session: &mut Session) {
        let Some(scenario) = self.pending else {
            return;
        };
        let Some(player) = host.players().server_players().into_iter().next() else {
            return;
        };
        if scenario.needs_session() && session.slot(player.uuid()).is_none() {
            return;
        }
        if scenario.needs_vial() && vial_in_slot0(host, player).is_none() {
            return;
        }
        self.pending = None;
        let fields = match scenario {
            Scenario::PdRoundtrip => pd_roundtrip(host, player),
            Scenario::TxAbort => tx_abort(host, player),
            Scenario::RawRoot => raw_root(host, player),
            Scenario::Report => report(host, session, player),
            Scenario::Drink => drink(host, session, player, VIAL_MILLI, false, false),
            Scenario::DrinkFail => drink(host, session, player, VIAL_MILLI, true, false),
            Scenario::Burn => drink(host, session, player, VIAL_MILLI, false, true),
            Scenario::BurnSmall => drink(host, session, player, SMALL_MILLI, false, true),
            Scenario::CorruptCrc => corrupt(host, session, player, Corruption::Crc),
            Scenario::FutureSchema => corrupt(host, session, player, Corruption::FutureSchema),
        };
        eprintln!("MISTBRN_TEST scenario={} {fields}", scenario.name());
    }
}

fn vial_in_slot0(host: &dyn Host, player: PlayerRef) -> Option<ItemStack> {
    let key = host.item_key(&Identifier::parse(IRON_VIAL).ok()?)?;
    player
        .inventory_slots(host)?
        .into_iter()
        .find(|(slot, stack)| *slot == 0 && stack.item() == key)
        .map(|(_, stack)| stack)
}

fn item_count(host: &dyn Host, player: PlayerRef, id: &str) -> u32 {
    let Some(key) = Identifier::parse(id).ok().and_then(|id| host.item_key(&id)) else {
        return 0;
    };
    player.inventory_slots(host).map_or(0, |slots| {
        slots
            .iter()
            .filter(|(_, stack)| stack.item() == key)
            .map(|(_, stack)| stack.count())
            .sum()
    })
}

fn fnv64(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// The UUID as the game names player files: `8-4-4-4-12` lowercase hex.
fn dashed_uuid(uuid: u128) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        uuid >> 96,
        (uuid >> 80) & 0xffff,
        (uuid >> 64) & 0xffff,
        (uuid >> 48) & 0xffff,
        uuid & 0xffff_ffff_ffff
    )
}

fn burn_name(burn: BurnState) -> &'static str {
    match burn {
        BurnState::Off => "off",
        BurnState::Burning => "burning",
        BurnState::Flared => "flared",
    }
}

/// The P5 progress of a save, for reports: experience, points, nodes, research, rules.
pub(crate) fn progress_fields(saved: &SavedPlayer) -> String {
    use allomancy_core::{CATALOG, Pair, Pool};
    let progress = saved.progress();
    let nodes: Vec<String> = progress
        .nodes()
        .iter()
        .map(|(code, rank)| format!("{code}:{rank}"))
        .collect();
    let research: Vec<String> = progress
        .all_research()
        .iter()
        .map(|(code, r)| {
            format!(
                "{code}:e{}h{}c{}",
                r.evidence,
                r.hints,
                u8::from(r.concluded)
            )
        })
        .collect();
    // The windows of the catalog's categories only: the reserved ones (schema 4) stay unopened.
    let windows: Vec<String> = CATALOG
        .practice
        .iter()
        .map(|def| progress.windows()[usize::from(def.category)])
        .map(|w| format!("{}/{}", w.start, w.count))
        .collect();
    format!(
        "xp_iron_steel={} pair_points={} knowledge={} nodes={} research={} windows={} selected_pair={} capacity_iron={} reduction={} milestones={} unlocked_pairs={}",
        progress.pair_xp(Pair::IronSteel),
        progress.available(Pool::Pair(Pair::IronSteel), &CATALOG),
        progress.available(Pool::Knowledge, &CATALOG),
        if nodes.is_empty() {
            "-".into()
        } else {
            nodes.join(",")
        },
        if research.is_empty() {
            "-".into()
        } else {
            research.join(",")
        },
        windows.join(","),
        crate::controls::pair_name(saved.state().selected_pair),
        saved.rules().capacity(Metal::Iron).unwrap_or(0),
        saved.rules().reduction_milli,
        progress.milestones(),
        unlocked_pairs(saved),
    )
}

/// The pairs whose two metals are unlocked, comma-separated (`-` for none).
fn unlocked_pairs(saved: &SavedPlayer) -> String {
    let pairs: Vec<&str> = allomancy_core::PAIRS
        .into_iter()
        .filter(|pair| {
            pair.metals()
                .iter()
                .all(|metal| saved.state().metals[metal.index()].unlocked)
        })
        .map(crate::controls::pair_name)
        .collect();
    if pairs.is_empty() {
        "-".into()
    } else {
        pairs.join(",")
    }
}

fn report(host: &mut dyn ServerHost, session: &Session, player: PlayerRef) -> String {
    let read: &dyn Host = host;
    let raw = save::raw_bytes(read, player).map_or("none".into(), |b| fnv64(&b));
    let dimension = player
        .dimension(read)
        .map_or("none".into(), |d| d.to_string());
    let schema = save::raw_bytes(read, player)
        .and_then(|b| allomancy_core::save_schema(&b))
        .map_or("none".into(), |v| v.to_string());
    let common = format!(
        "uuid={} vials={} empty_vials={} primers={} notebooks={} basic_vials={} dimension={dimension} raw_hash={raw} schema={schema}",
        dashed_uuid(player.uuid()),
        item_count(read, player, IRON_VIAL),
        item_count(read, player, EMPTY_VIAL),
        item_count(read, player, UNLOCK_ITEMS[0].0),
        item_count(read, player, crate::content::NOTEBOOK),
        item_count(read, player, allomancy_core::BASIC_VIAL)
    );
    match session.slot(player.uuid()) {
        Some(Slot::Blocked(blocked)) => {
            format!("state=blocked reason={} {common}", blocked.reason())
        }
        Some(Slot::Active { saved, .. }) => {
            let iron = saved.state().metals[Metal::Iron.index()];
            let steel = saved.state().metals[Metal::Steel.index()];
            let progress = progress_fields(saved);
            format!(
                "state=active reserve_iron={} burn_iron={} unlocked_iron={} unlocked_steel={} burn_steel={} reserve_steel={} reserve_tin={} remainder_iron={} revision={} last_transaction={} {progress} {common}",
                iron.reserve_milli,
                burn_name(iron.burn),
                iron.unlocked,
                steel.unlocked,
                burn_name(steel.burn),
                steel.reserve_milli,
                saved.state().metals[Metal::Tin.index()].reserve_milli,
                saved.accounting().remainder[Metal::Iron.index()],
                saved.revision(),
                saved.last_transaction(),
            )
        }
        None => format!("state=unloaded {common}"),
    }
}

fn drink(
    host: &mut dyn ServerHost,
    session: &mut Session,
    player: PlayerRef,
    amount: i64,
    fail_inside: bool,
    burn_after: bool,
) -> String {
    let read: &dyn Host = host;
    let Some(stack) = vial_in_slot0(read, player) else {
        return "error=no_vial".into();
    };
    let uuid = player.uuid();
    if let Err(error) = session.change(uuid, true, |saved| {
        update_player(saved, |state, _| {
            state.metals[Metal::Iron.index()].unlocked = true;
            Ok(())
        })
    }) {
        return format!("error=unlock:{error}");
    }
    session.flush(host, player);
    let result = match session.credit_from_slot(
        host,
        Drink {
            player,
            slot: 0,
            stack,
            metal: Some(Metal::Iron),
        },
        amount,
        fail_inside,
    ) {
        Ok((seq, unlocked)) => format!("result=ok seq={} vial_unlocked={unlocked}", seq.0),
        Err(error) => format!("result=err:{error}"),
    };
    if burn_after && result.starts_with("result=ok") {
        let burned = session.change(uuid, true, |saved| {
            update_player(saved, |state, _| {
                state.metals[Metal::Iron.index()].burn = BurnState::Burning;
                Ok(())
            })
        });
        session.flush(host, player);
        if let Err(error) = burned {
            return format!("{result} burn=err:{error}");
        }
        eprintln!(
            "MISTBRN event=burn_started metal={}",
            metal_name(Metal::Iron)
        );
    }
    format!("{result} {}", report(host, session, player))
}

enum Corruption {
    Crc,
    FutureSchema,
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn corrupt(
    host: &mut dyn ServerHost,
    session: &mut Session,
    player: PlayerRef,
    kind: Corruption,
) -> String {
    let Some(saved) = session.saved(player.uuid()).cloned() else {
        return "error=not_active".into();
    };
    let Ok(mut bytes) = encode_save(&saved) else {
        return "error=encode".into();
    };
    let reason = match kind {
        Corruption::Crc => {
            *bytes.last_mut().expect("encoded saves are never empty") ^= 0xFF;
            Blocked::Decode(SaveError::ChecksumMismatch)
        }
        Corruption::FutureSchema => {
            let future = allomancy_core::SAVE_SCHEMA + 1;
            bytes[4..6].copy_from_slice(&future.to_le_bytes());
            let body = bytes.len() - 4;
            let crc = crc32(&bytes[..body]);
            bytes[body..].copy_from_slice(&crc.to_le_bytes());
            Blocked::Decode(SaveError::UnsupportedVersion(future))
        }
    };
    let hash = fnv64(&bytes);
    if let Err(error) = save::write_bytes(host, player, bytes) {
        return format!("error=write:{error}");
    }
    // From here on this session must not overwrite what it just planted.
    session.block(host, player.uuid(), reason);
    format!("written=true raw_hash={hash}")
}

fn persistent(host: &dyn Host, player: PlayerRef) -> Option<Compound> {
    player.persistent_data(host)
}

fn persisted_sub(data: &Compound) -> Option<&Compound> {
    match data.get(&PERSISTED.into()) {
        Some(Tag::Compound(sub)) => Some(sub),
        _ => None,
    }
}

/// `data` with `key` set under `PlayerPersisted`, every other key kept.
fn with_persisted(mut data: Compound, key: &str, value: Tag) -> Compound {
    let mut sub = persisted_sub(&data).cloned().unwrap_or_default();
    sub.insert(key.into(), value);
    data.insert(PERSISTED.into(), Tag::Compound(sub));
    data
}

fn pd_roundtrip(host: &mut dyn ServerHost, player: PlayerRef) -> String {
    let Some(data) = persistent(host, player) else {
        return "error=no_persistent_data".into();
    };
    match persisted_sub(&data).and_then(|sub| sub.get(&PROBE_KEY.into())) {
        Some(Tag::ByteArray(bytes)) => format!(
            "phase=read value={}",
            bytes
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Some(_) => "phase=read error=wrong_type".into(),
        None => {
            let next = with_persisted(data, PROBE_KEY, Tag::ByteArray(PROBE_BYTES.to_vec()));
            match player.set_persistent_data(host, next) {
                Ok(()) => "phase=write ok=true".into(),
                Err(error) => format!("phase=write ok=false error={error:?}"),
            }
        }
    }
}

fn slot0_count(host: &dyn Host, player: PlayerRef) -> Option<u32> {
    let slots = player.inventory_slots(host)?;
    Some(
        slots
            .iter()
            .find(|(slot, _)| *slot == 0)
            .map_or(0, |(_, stack)| stack.count()),
    )
}

fn tx_abort(host: &mut dyn ServerHost, player: PlayerRef) -> String {
    let (Some(count_before), Some(data_before)) =
        (slot0_count(host, player), persistent(host, player))
    else {
        return "error=unreadable_player".into();
    };
    if count_before < 2 {
        return format!("error=need_two_items_in_slot0 count={count_before}");
    }
    let mut inside_count = None;
    let mut inside_pd = None;
    let result: Result<(), String> = host.transaction(|host, _tx| {
        let read: &dyn Host = host;
        let stack = player
            .inventory_slots(read)
            .and_then(|slots| slots.into_iter().find(|(slot, _)| *slot == 0))
            .map(|(_, stack)| stack)
            .ok_or("slot0_empty")?;
        let shrunk = stack.shrink(1);
        player
            .set_inventory_slot(host, 0, shrunk)
            .map_err(|e| format!("set_slot:{e:?}"))?;
        let data = persistent(host, player).ok_or("no_persistent_data")?;
        player
            .set_persistent_data(host, with_persisted(data, ABORT_KEY, Tag::Byte(1)))
            .map_err(|e| format!("set_pd:{e:?}"))?;
        inside_count = slot0_count(host, player);
        // Prove the write took effect inside the transaction, so `pd_unchanged` means rollback.
        inside_pd = persistent(host, player).and_then(|data| {
            match persisted_sub(&data)?.get(&ABORT_KEY.into()) {
                Some(Tag::Byte(value)) => Some(*value),
                _ => None,
            }
        });
        Err("deliberate_abort".into())
    });
    let count_after = slot0_count(host, player);
    let data_after = persistent(host, player);
    format!(
        "result={} count_before={count_before} inside_count={} inside_pd={} count_after={} pd_unchanged={}",
        result.err().unwrap_or_else(|| "committed".into()),
        inside_count.map_or("none".into(), |c| c.to_string()),
        inside_pd.map_or("none".into(), |v| v.to_string()),
        count_after.map_or("none".into(), |c| c.to_string()),
        data_after.as_ref() == Some(&data_before)
    )
}

fn raw_root(host: &mut dyn ServerHost, player: PlayerRef) -> String {
    // Make sure there is something to find, then look for it in the raw entity data.
    let Some(data) = persistent(host, player) else {
        return "error=no_persistent_data".into();
    };
    let marked = with_persisted(data, PROBE_KEY, Tag::ByteArray(PROBE_BYTES.to_vec()));
    if let Err(error) = player.set_persistent_data(host, marked) {
        return format!("error=set_pd:{error:?}");
    }
    match player.entity().nbt(host) {
        None => "raw_nbt=none".into(),
        Some(nbt) => match nbt.get(&"NeoForgeData".into()) {
            Some(Tag::Compound(_)) => "raw_nbt=some has_neoforgedata=true type=compound".into(),
            Some(_) => "raw_nbt=some has_neoforgedata=true type=other".into(),
            None => "raw_nbt=some has_neoforgedata=false".into(),
        },
    }
}
