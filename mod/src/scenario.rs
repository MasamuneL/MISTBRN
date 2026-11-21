//! In-game test scenarios, run only when `MISTBRN_TEST_SCENARIO` names one.
//!
//! Without the variable the runner is inert: no reads, writes or log lines. Each scenario runs
//! once per process, on the first server tick with a player online, and reports
//! `MISTBRN_TEST scenario=<name> key=value ...` for `tools/native-test.sh`.
use mod_api::{Compound, Host, HostTransaction, PlayerRef, ServerHost, Tag};

const PERSISTED: &str = "PlayerPersisted";
const PROBE_KEY: &str = "mistbrn_probe";
const ABORT_KEY: &str = "mistbrn_abort_probe";
const PROBE_BYTES: [u8; 3] = [1, 2, 3];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scenario {
    /// G5: write a probe into `NeoForgeData.PlayerPersisted`, read it back after a restart.
    PdRoundtrip,
    /// G6: a transaction that writes inventory and persistent data, then aborts.
    TxAbort,
    /// G7: does the player's raw entity NBT expose `NeoForgeData`?
    RawRoot,
}

impl Scenario {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "pd-roundtrip" => Some(Self::PdRoundtrip),
            "tx-abort" => Some(Self::TxAbort),
            "raw-root" => Some(Self::RawRoot),
            _ => None,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::PdRoundtrip => "pd-roundtrip",
            Self::TxAbort => "tx-abort",
            Self::RawRoot => "raw-root",
        }
    }
}

pub(crate) struct Runner {
    pending: Option<Scenario>,
}

impl Runner {
    pub(crate) fn from_env() -> Self {
        let pending = std::env::var("MISTBRN_TEST_SCENARIO")
            .ok()
            .filter(|name| !name.is_empty())
            .and_then(|name| {
                let parsed = Scenario::parse(&name);
                if parsed.is_none() {
                    eprintln!("MISTBRN_TEST scenario={name} error=unknown_scenario");
                }
                parsed
            });
        Self { pending }
    }

    pub(crate) fn tick(&mut self, host: &mut dyn ServerHost) {
        let Some(scenario) = self.pending else {
            return;
        };
        let Some(player) = host.players().server_players().into_iter().next() else {
            return;
        };
        self.pending = None;
        let fields = match scenario {
            Scenario::PdRoundtrip => pd_roundtrip(host, player),
            Scenario::TxAbort => tx_abort(host, player),
            Scenario::RawRoot => raw_root(host, player),
        };
        eprintln!("MISTBRN_TEST scenario={} {fields}", scenario.name());
    }
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
