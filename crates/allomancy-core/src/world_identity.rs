//! Immutable, offline-provisioned world identity. Reading never repairs a marker.
use crate::WorldId;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;

pub const MAX_WORLD_IDENTITY_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldIdentitySource {
    Primary,
    Backup,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedWorldIdentity {
    pub world_id: WorldId,
    pub source: WorldIdentitySource,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldIdentityError {
    Parse,
    Schema,
    Future(u64),
    InvalidId,
    Conflict,
    Io,
    TooLarge,
}
impl std::fmt::Display for WorldIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for WorldIdentityError {}

// Value alone erases duplicate fields; deserialize a map with explicit uniqueness.
struct Marker(BTreeMap<String, Value>);
impl<'de> Deserialize<'de> for Marker {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Object;
        impl<'de> Visitor<'de> for Object {
            type Value = Marker;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("world identity object")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Marker, M::Error> {
                let mut fields = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, Value>()? {
                    if fields.insert(key, value).is_some() {
                        return Err(serde::de::Error::custom("duplicate marker field"));
                    }
                }
                Ok(Marker(fields))
            }
        }
        deserializer.deserialize_map(Object)
    }
}

pub fn decode_world_identity(bytes: &[u8]) -> Result<WorldId, WorldIdentityError> {
    use WorldIdentityError as E;
    if bytes.len() > MAX_WORLD_IDENTITY_BYTES {
        return Err(E::TooLarge);
    }
    std::str::from_utf8(bytes).map_err(|_| E::Parse)?;
    let Marker(fields) = serde_json::from_slice(bytes).map_err(|error| {
        if error.is_syntax() || error.is_eof() {
            E::Parse
        } else {
            E::Schema
        }
    })?;
    let version = fields
        .get("schema_version")
        .and_then(Value::as_u64)
        .ok_or(E::Schema)?;
    if version > 1 {
        return Err(E::Future(version));
    }
    if version != 1 || fields.len() != 2 {
        return Err(E::Schema);
    }
    let id = fields
        .get("world_id")
        .and_then(Value::as_str)
        .ok_or(E::Schema)?;
    if id.len() != 32
        || !id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(E::InvalidId);
    }
    Ok(WorldId::new(id))
}

fn read_marker(path: &Path) -> Result<Option<WorldId>, WorldIdentityError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(WorldIdentityError::Io),
    };
    let mut bytes = Vec::new();
    file.take(MAX_WORLD_IDENTITY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| WorldIdentityError::Io)?;
    decode_world_identity(&bytes).map(Some)
}

/// `world_root` is the world directory, not the marker filename.
/// A valid backup is reported explicitly; no recovery write occurs.
pub fn load_world_identity(
    world_root: &Path,
) -> Result<Option<LoadedWorldIdentity>, WorldIdentityError> {
    let directory = world_root.join("allomancy");
    let primary = read_marker(&directory.join("world-identity.json"))?;
    let backup = read_marker(&directory.join("world-identity.json.bak"))?;
    if let (Some(a), Some(b)) = (&primary, &backup) {
        if a != b {
            return Err(WorldIdentityError::Conflict);
        }
    }
    Ok(match (primary, backup) {
        (Some(world_id), _) => Some(LoadedWorldIdentity {
            world_id,
            source: WorldIdentitySource::Primary,
        }),
        (None, Some(world_id)) => Some(LoadedWorldIdentity {
            world_id,
            source: WorldIdentitySource::Backup,
        }),
        (None, None) => None,
    })
}
