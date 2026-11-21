//! Where a player's save lives in the game: `NeoForgeData.PlayerPersisted.mistbrn` (ByteArray).
//!
//! `PlayerPersisted` survives death respawns and the first End exit; inventory and NeoForgeData
//! share the player's file and the same `host.transaction` journal. Reads never repair: anything
//! unexpected blocks the player, and a blocked player's data is never written.
use allomancy_core::{PlayerId, SaveError, SavedPlayer, WorldId, decode_save, encode_save};
use mod_api::{Compound, Host, PlayerRef, ServerHost, Tag};

pub(crate) const NEOFORGE_DATA: &str = "NeoForgeData";
pub(crate) const PLAYER_PERSISTED: &str = "PlayerPersisted";
pub(crate) const SAVE_KEY: &str = "mistbrn";

/// Why a player's save cannot be used. Each is reported once and blocks writes for the session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Blocked {
    /// `NeoForgeData` in the raw player NBT is not a compound (the getter would hide it).
    RootNotCompound,
    /// `NeoForgeData.PlayerPersisted` is not a compound.
    PersistedNotCompound,
    /// `PlayerPersisted.mistbrn` is not a ByteArray.
    KeyNotByteArray,
    /// The bytes did not decode.
    Decode(SaveError),
}

impl Blocked {
    pub(crate) fn reason(&self) -> String {
        match self {
            Self::RootNotCompound => "root_not_compound".into(),
            Self::PersistedNotCompound => "persisted_not_compound".into(),
            Self::KeyNotByteArray => "key_not_bytearray".into(),
            Self::Decode(SaveError::ChecksumMismatch) => "crc".into(),
            Self::Decode(SaveError::UnsupportedVersion(v)) => format!("unsupported_version:{v}"),
            Self::Decode(error) => format!("decode:{error:?}"),
        }
    }
}

/// What a read found (short-lived; never stored, so no boxing).
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Loaded {
    /// No save yet: a fresh one may be created and written.
    Absent,
    Found(SavedPlayer),
    Blocked(Blocked),
}

/// The player is not readable right now (stale handle, swapped out): try again later.
#[derive(Debug)]
pub(crate) struct Unavailable;

/// Reads and validates the player's save. The raw NBT is checked first because
/// `persistent_data()` turns a non-compound `NeoForgeData` into an empty compound.
pub(crate) fn read(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    world: &WorldId,
    id: &PlayerId,
) -> Result<Loaded, Unavailable> {
    let raw = player.entity().nbt(host).ok_or(Unavailable)?;
    if let Some(root) = raw.get(&NEOFORGE_DATA.into())
        && !matches!(root, Tag::Compound(_))
    {
        return Ok(Loaded::Blocked(Blocked::RootNotCompound));
    }
    let read: &dyn Host = host;
    let data = player.persistent_data(read).ok_or(Unavailable)?;
    let sub = match data.get(&PLAYER_PERSISTED.into()) {
        None => return Ok(Loaded::Absent),
        Some(Tag::Compound(sub)) => sub,
        Some(_) => return Ok(Loaded::Blocked(Blocked::PersistedNotCompound)),
    };
    match sub.get(&SAVE_KEY.into()) {
        None => Ok(Loaded::Absent),
        Some(Tag::ByteArray(bytes)) => Ok(match decode_save(bytes, world, id) {
            Ok(saved) => Loaded::Found(saved),
            Err(error) => Loaded::Blocked(Blocked::Decode(error)),
        }),
        Some(_) => Ok(Loaded::Blocked(Blocked::KeyNotByteArray)),
    }
}

/// The raw save bytes, unvalidated (diagnostics and tests only).
pub(crate) fn raw_bytes(host: &dyn Host, player: PlayerRef) -> Option<Vec<u8>> {
    let data = player.persistent_data(host)?;
    match data.get(&PLAYER_PERSISTED.into())? {
        Tag::Compound(sub) => match sub.get(&SAVE_KEY.into())? {
            Tag::ByteArray(bytes) => Some(bytes.clone()),
            _ => None,
        },
        _ => None,
    }
}

/// `data` with `PlayerPersisted.mistbrn = bytes`, every other key and namespace kept.
/// Refuses (returns None) if `PlayerPersisted` exists and is not a compound.
pub(crate) fn with_save_bytes(mut data: Compound, bytes: Vec<u8>) -> Option<Compound> {
    let mut sub = match data.remove(&PLAYER_PERSISTED.into()) {
        None => Compound::new(),
        Some(Tag::Compound(sub)) => sub,
        Some(_) => return None,
    };
    sub.insert(SAVE_KEY.into(), Tag::ByteArray(bytes));
    data.insert(PLAYER_PERSISTED.into(), Tag::Compound(sub));
    Some(data)
}

/// Writes raw bytes as the player's save (in the open transaction, if any).
pub(crate) fn write_bytes(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    bytes: Vec<u8>,
) -> Result<(), String> {
    let read: &dyn Host = host;
    let data = player.persistent_data(read).ok_or("unavailable")?;
    let next = with_save_bytes(data, bytes).ok_or("persisted_not_compound")?;
    player
        .set_persistent_data(host, next)
        .map_err(|error| format!("{error:?}"))
}

/// Encodes and writes `saved` (in the open transaction, if any).
pub(crate) fn write(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    saved: &SavedPlayer,
) -> Result<(), String> {
    let bytes = encode_save(saved).map_err(|error| format!("encode:{error:?}"))?;
    write_bytes(host, player, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The compound's key type, whatever the SDK names it.
    type K = <Compound as KeyType>::Key;
    trait KeyType {
        type Key;
    }
    impl<K, V> KeyType for std::collections::BTreeMap<K, V> {
        type Key = K;
    }

    // Catches a write that drops other mods' keys or the rest of PlayerPersisted.
    #[test]
    fn writing_keeps_every_other_key() {
        let mut persisted = Compound::new();
        persisted.insert(K::from("other_mod"), Tag::Int(7));
        let mut data = Compound::new();
        data.insert(K::from("Foreign"), Tag::Byte(1));
        data.insert(K::from(PLAYER_PERSISTED), Tag::Compound(persisted));
        let next = with_save_bytes(data, vec![1, 2]).expect("compound PlayerPersisted");
        assert_eq!(next.get(&K::from("Foreign")), Some(&Tag::Byte(1)));
        let Some(Tag::Compound(sub)) = next.get(&K::from(PLAYER_PERSISTED)) else {
            panic!("PlayerPersisted kept as a compound");
        };
        assert_eq!(sub.get(&K::from("other_mod")), Some(&Tag::Int(7)));
        assert_eq!(
            sub.get(&K::from(SAVE_KEY)),
            Some(&Tag::ByteArray(vec![1, 2]))
        );
    }

    #[test]
    fn writing_creates_player_persisted_when_absent() {
        let next = with_save_bytes(Compound::new(), vec![9]).expect("absent is fine");
        let Some(Tag::Compound(sub)) = next.get(&K::from(PLAYER_PERSISTED)) else {
            panic!("created");
        };
        assert_eq!(sub.get(&K::from(SAVE_KEY)), Some(&Tag::ByteArray(vec![9])));
    }

    // Catches overwriting a PlayerPersisted of the wrong type (someone else's data).
    #[test]
    fn writing_refuses_a_non_compound_player_persisted() {
        let mut data = Compound::new();
        data.insert(K::from(PLAYER_PERSISTED), Tag::Int(3));
        assert!(with_save_bytes(data, vec![1]).is_none());
    }

    #[test]
    fn block_reasons_are_stable_for_logs_and_tests() {
        assert_eq!(Blocked::RootNotCompound.reason(), "root_not_compound");
        assert_eq!(Blocked::Decode(SaveError::ChecksumMismatch).reason(), "crc");
        assert_eq!(
            Blocked::Decode(SaveError::UnsupportedVersion(2)).reason(),
            "unsupported_version:2"
        );
    }
}
