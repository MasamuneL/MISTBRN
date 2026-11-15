// Read-only gate injected before WorldRequest::prepare_world's original body.
// A .dat backup never silently supplies economy over another inventory snapshot.
use crate::save::{Level, uuid_from_text, uuid_text};
use allomancy_core::{PlayerId, WorldId, WorldIdentitySource, decode_save, load_world_identity};
use horizon_world::{
    compression::Compression,
    nbt::{self, JavaString, Limits, Tag},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{ErrorKind, Read},
    path::{Path, PathBuf},
    sync::Mutex,
};

type GuardResult<T> = std::result::Result<T, String>;
const MAX_DAT_BYTES: usize = 64 * 1024 * 1024;
const MAX_NODES: usize = 1_000_000;
const MAX_DEPTH: usize = 128;
const MAX_PLAYER_FILES: usize = 4096;
fn limits() -> Limits {
    Limits {
        max_bytes: MAX_DAT_BYTES,
        max_nodes: MAX_NODES,
        max_depth: MAX_DEPTH,
    }
}
fn fail(reason: &str) -> String {
    format!("Allomancy preflight: {reason}")
}

pub enum OwnerMode {
    Integrated,
    Dedicated,
    Unsupported,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OwnerAdmission {
    Vanilla,
    Fresh,
    Existing,
    Dedicated,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct OwnerContext {
    owner: Option<[i32; 4]>,
    admission: Option<OwnerAdmission>,
    initialized: bool,
}

// Startup context only: no economic state, durable identity or automatic reset.
static OWNERS: Mutex<BTreeMap<PathBuf, OwnerContext>> = Mutex::new(BTreeMap::new());

fn protected(world: &Path) -> bool {
    ["allomancy/world-identity.json", "allomancy/world-identity.json.bak"]
        .iter()
        .any(|name| !matches!(fs::symlink_metadata(world.join(name)), Err(e) if e.kind() == ErrorKind::NotFound))
}

fn world_key(world: &Path) -> GuardResult<PathBuf> {
    let root = fs::canonicalize(world).map_err(|_| fail("owner root canonicalization failed"))?;
    if !fs::metadata(&root)
        .map_err(|_| fail("owner root metadata failed"))?
        .is_dir()
    {
        return Err(fail("owner root is not a directory"));
    }
    Ok(root)
}

fn context(root: &Path) -> GuardResult<OwnerContext> {
    OWNERS
        .lock()
        .map_err(|_| fail("owner context poisoned"))?
        .get(root)
        .copied()
        .ok_or_else(|| fail("owner context missing"))
}

pub fn capture_owner(world: &Path, owner: [i32; 4]) -> GuardResult<()> {
    if !protected(world) {
        return Ok(());
    }
    let root = world_key(world)?;
    let mut owners = OWNERS.lock().map_err(|_| fail("owner context poisoned"))?;
    if let Some(current) = owners.get(&root) {
        if current.owner != Some(owner) {
            return Err(fail("owner context conflict"));
        }
    } else {
        owners.insert(
            root,
            OwnerContext {
                owner: Some(owner),
                admission: None,
                initialized: false,
            },
        );
    }
    drop(owners);
    log::info!("Allomancy owner: context captured");
    Ok(())
}

// Audit before requiring context: vanilla retains its original owner policy.
pub fn validate_world(world: &Path, mode: OwnerMode) -> GuardResult<()> {
    let admission = audit_world(world, None)?;
    if admission == OwnerAdmission::Vanilla {
        return Ok(());
    }
    let root = world_key(world)?;
    if matches!(mode, OwnerMode::Dedicated) {
        let dedicated = OwnerContext {
            owner: None,
            admission: Some(OwnerAdmission::Dedicated),
            initialized: false,
        };
        let mut owners = OWNERS.lock().map_err(|_| fail("owner context poisoned"))?;
        if owners
            .get(&root)
            .is_some_and(|current| *current != dedicated)
        {
            return Err(fail("owner context mode conflict"));
        }
        owners.entry(root).or_insert(dedicated);
        drop(owners);
        log::info!("Allomancy owner: dedicated admitted");
        return Ok(());
    }
    if !matches!(mode, OwnerMode::Integrated) {
        return Err(fail("unsupported protected world role"));
    }
    let before = context(&root)?;
    let expected = before
        .owner
        .ok_or_else(|| fail("integrated owner context missing"))?;
    if admission == OwnerAdmission::Existing
        && audit_world(world, Some(expected))? != OwnerAdmission::Existing
    {
        return Err(fail("owner admission changed during audit"));
    }
    let mut owners = OWNERS.lock().map_err(|_| fail("owner context poisoned"))?;
    let current = owners
        .get_mut(&root)
        .ok_or_else(|| fail("owner context missing"))?;
    if *current != before {
        return Err(fail("owner context changed during audit"));
    }
    match (current.admission, admission, current.initialized) {
        (None, _, _) | (Some(OwnerAdmission::Fresh), OwnerAdmission::Existing, _) => {
            current.admission = Some(admission);
            current.initialized = admission == OwnerAdmission::Existing;
        }
        (Some(OwnerAdmission::Fresh), OwnerAdmission::Fresh, false)
        | (Some(OwnerAdmission::Existing), OwnerAdmission::Existing, _) => {}
        _ => return Err(fail("owner admission cannot reset")),
    }
    drop(owners);
    log::info!("Allomancy owner: integrated admitted");
    Ok(())
}

pub fn before_level_write(world: &Path, level: &mut Level) -> GuardResult<()> {
    if !protected(world) {
        return Ok(());
    }
    let root = world_key(world)?;
    let before = context(&root)?;
    match before.admission {
        Some(OwnerAdmission::Dedicated) if before.owner.is_none() => return Ok(()),
        Some(OwnerAdmission::Fresh | OwnerAdmission::Existing) => {}
        _ => return Err(fail("protected write has no admission")),
    }
    let expected = before
        .owner
        .ok_or_else(|| fail("integrated owner context missing"))?;
    if before.initialized || before.admission == Some(OwnerAdmission::Existing) {
        return if level.player_uuid == Some(expected) {
            Ok(())
        } else {
            Err(fail("owner UUID changed before write"))
        };
    }
    // Only the first Fresh write rereads DAT, before a player writer can exist.
    if audit_world(world, Some(expected))? != OwnerAdmission::Fresh
        || level.player_uuid.is_some_and(|id| id != expected)
    {
        return Err(fail("owner bootstrap context mismatch"));
    }
    let mut owners = OWNERS.lock().map_err(|_| fail("owner context poisoned"))?;
    let current = owners
        .get_mut(&root)
        .ok_or_else(|| fail("owner context missing"))?;
    if *current != before {
        return Err(fail("owner context changed before bootstrap"));
    }
    level.player_uuid = Some(expected);
    current.initialized = true;
    drop(owners);
    log::info!("Allomancy owner: bootstrap accepted in memory; durable=false");
    Ok(())
}

fn audit_world(world: &Path, expected_owner: Option<[i32; 4]>) -> GuardResult<OwnerAdmission> {
    let identity = load_world_identity(world).map_err(|e| fail(&format!("world identity {e}")))?;
    if identity
        .as_ref()
        .is_some_and(|i| i.source == WorldIdentitySource::Backup)
    {
        log::warn!("Allomancy preflight: identity source Backup; no marker was rewritten");
    }
    let id = identity.as_ref().map(|i| &i.world_id);
    let primary = read_dat(&world.join("level.dat"))?;
    let backup = read_dat(&world.join("level.dat_old"))?;
    let mut owner = None;
    let mut legacy_owner = false;
    for (index, root) in [primary.as_ref(), backup.as_ref()].into_iter().enumerate() {
        if let Some(root) = root {
            let data = root
                .get("Data")
                .filter(|t| matches!(t, Tag::Compound(_)))
                .ok_or_else(|| fail("level Data must be Compound"))?;
            let named = data.get("singleplayer_uuid").map(uuid).transpose()?;
            if let Some(player) = data.get("Player") {
                validate_player(player, id, named)?;
            }
            if index == 0 {
                owner = named;
                legacy_owner = data.get("Player").is_some();
            }
        }
    }
    if primary.is_none() && backup.is_some() {
        return Err(fail(
            "level primary missing; restore a complete snapshot offline",
        ));
    }
    let directory = world.join("players/data");
    let entries = match fs::symlink_metadata(&directory) {
        Ok(meta) if meta.file_type().is_dir() => {
            Some(fs::read_dir(&directory).map_err(|_| fail("player directory IO error"))?)
        }
        Ok(_) => return Err(fail("player data path is not a regular directory")),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(_) => return Err(fail("player directory metadata IO error")),
    };
    let mut primaries = BTreeMap::new();
    let mut backups = BTreeSet::new();
    let mut count = 0;
    if let Some(entries) = entries {
        for entry in entries {
            let entry = entry.map_err(|_| fail("player directory entry IO error"))?;
            let filename = entry.file_name();
            let name = filename
                .to_str()
                .ok_or_else(|| fail("non-UTF8 player filename"))?;
            // Windows resolves these extensions case-insensitively too. Never
            // skip a .DAT that the engine can subsequently open as .dat.
            let lower_name = name.to_ascii_lowercase();
            let (stem, old) = if let Some(stem) = lower_name.strip_suffix(".dat_old") {
                (&name[..stem.len()], true)
            } else if let Some(stem) = lower_name.strip_suffix(".dat") {
                (&name[..stem.len()], false)
            } else {
                continue;
            };
            count += 1;
            if count > MAX_PLAYER_FILES {
                return Err(fail("player file limit exceeded"));
            }
            let expected =
                uuid_from_text(stem).ok_or_else(|| fail("invalid player filename UUID"))?;
            if uuid_text(expected) != stem {
                return Err(fail("noncanonical player filename UUID"));
            }
            let root = read_dat(&entry.path())?
                .ok_or_else(|| fail("player file disappeared during validation"))?;
            validate_player(&root, id, Some(expected))?;
            if old {
                backups.insert(stem.to_owned());
            } else {
                primaries.insert(stem.to_owned(), root.get("UUID").map(uuid).transpose()?);
            }
        }
    }
    if backups.iter().any(|name| !primaries.contains_key(name)) {
        return Err(fail(
            "player primary missing; restore a complete snapshot offline",
        ));
    }
    if primary.is_none() && count != 0 {
        return Err(fail("player data exists without level primary"));
    }
    if let Some(owner) = owner {
        if !primaries.contains_key(&uuid_text(owner)) && !legacy_owner {
            return Err(fail("level names a missing owner snapshot"));
        }
    }
    let admission = if id.is_none() {
        OwnerAdmission::Vanilla
    } else if primary.is_none() && backup.is_none() && count == 0 {
        log::info!("Allomancy owner: fresh snapshots absent");
        OwnerAdmission::Fresh
    } else {
        if let Some(expected) = expected_owner {
            for root in [primary.as_ref(), backup.as_ref()].into_iter().flatten() {
                validate_owner(root, &primaries, expected)?;
            }
            log::info!("Allomancy owner: existing snapshots accepted");
        }
        OwnerAdmission::Existing
    };
    log::info!(
        "Allomancy preflight: snapshot source Primary; backups audited only; identity {}",
        if id.is_some() {
            "present"
        } else {
            "absent (vanilla only)"
        }
    );
    Ok(admission)
}

// Mirror the identity preserved by Level::read, then load_owner's selection.
// A fallback file for B cannot initialize an existing Level UUID that is None.
fn validate_owner(
    root: &Tag,
    primaries: &BTreeMap<String, Option<[i32; 4]>>,
    expected: [i32; 4],
) -> GuardResult<()> {
    let data = root.get("Data").ok_or_else(|| fail("level Data missing"))?;
    let named = data.get("singleplayer_uuid").map(uuid).transpose()?;
    let named_file = named.and_then(|id| primaries.get(&uuid_text(id))).copied();
    let legacy = data
        .get("Player")
        .and_then(|player| player.get("UUID"))
        .map(uuid)
        .transpose()?;
    let level_uuid = match named_file {
        Some(stored) => stored.or(named),
        None => legacy.or(named),
    };
    let adopted_uuid = if named_file.is_some() || !primaries.contains_key(&uuid_text(expected)) {
        level_uuid
    } else {
        // load_owner prefers the own file when the named file was not loaded.
        // Its UUID/filename relationship has already passed strict validation.
        Some(expected)
    };
    if level_uuid != Some(expected) || adopted_uuid != Some(expected) {
        return Err(fail(
            "integrated owner identity mismatch or missing Level UUID",
        ));
    }
    Ok(())
}

fn uuid(tag: &Tag) -> GuardResult<[i32; 4]> {
    match tag {
        Tag::IntArray(v) if v.len() == 4 => Ok([v[0], v[1], v[2], v[3]]),
        Tag::String(s) => s
            .to_string()
            .ok()
            .and_then(|s| uuid_from_text(&s))
            .ok_or_else(|| fail("invalid snapshot UUID")),
        _ => Err(fail("invalid snapshot UUID tag")),
    }
}
fn validate_player(
    root: &Tag,
    world: Option<&WorldId>,
    expected: Option<[i32; 4]>,
) -> GuardResult<()> {
    if !matches!(root, Tag::Compound(_)) {
        return Err(fail("player root must be Compound"));
    }
    let stored = root.get("UUID").map(uuid).transpose()?;
    if stored.zip(expected).is_some_and(|(a, b)| a != b) {
        return Err(fail("player snapshot UUID mismatch"));
    }
    let Some(mods) = root.get("NeoForgeData") else {
        return Ok(());
    };
    if !matches!(mods, Tag::Compound(_)) {
        return Err(fail("NeoForgeData must be Compound"));
    }
    if mods.get("allomancy:storage_fault").is_some() {
        return Err(fail(
            "Allomancy storage fault present; restore a complete snapshot offline",
        ));
    }
    let Some(state) = mods.get("allomancy:state") else {
        return Ok(());
    };
    let world = world.ok_or_else(|| fail("Allomancy namespace exists without world identity"))?;
    let player = stored
        .or(expected)
        .ok_or_else(|| fail("Allomancy snapshot has no player UUID"))?;
    let Tag::Compound(fields) = state else {
        return Err(fail("Allomancy namespace must be Compound"));
    };
    match state.get("format_version") {
        Some(Tag::Int(1)) => {}
        Some(Tag::Int(v)) if *v > 1 => return Err(fail("future Allomancy transport version")),
        _ => return Err(fail("invalid Allomancy transport version")),
    }
    if fields.len() != 2 {
        return Err(fail("invalid Allomancy transport fields"));
    }
    let Some(Tag::ByteArray(bytes)) = state.get("payload") else {
        return Err(fail("Allomancy payload must be ByteArray"));
    };
    let player = PlayerId::new(uuid_text(player).replace('-', ""));
    decode_save(bytes, world, &player).map_err(|e| fail(&format!("Allomancy payload {e}")))?;
    Ok(())
}

fn read_dat(path: &Path) -> GuardResult<Option<Tag>> {
    match fs::symlink_metadata(path) {
        Ok(m) if !m.file_type().is_file() => return Err(fail("snapshot is not a regular file")),
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(fail("snapshot metadata IO error")),
    }
    let file = File::open(path).map_err(|_| fail("snapshot open IO error"))?;
    let mut bytes = Vec::new();
    file.take(MAX_DAT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| fail("snapshot read IO error"))?;
    if bytes.len() > MAX_DAT_BYTES {
        return Err(fail("snapshot byte limit exceeded"));
    }
    // Upstream accepts Gzip and raw NBT. Recognizable damaged Gzip must not be reinterpreted.
    let raw = if bytes.starts_with(&[0x1f, 0x8b]) {
        Compression::Gzip
            .decode(&bytes, MAX_DAT_BYTES)
            .map_err(|_| fail("snapshot Gzip decode error"))?
    } else {
        bytes
    };
    audit_named(&raw)?;
    let (named, consumed) =
        nbt::read_named(&raw, limits()).map_err(|_| fail("snapshot NBT decode error"))?;
    if consumed != raw.len() || !matches!(named.tag, Tag::Compound(_)) {
        return Err(fail("snapshot root/trailing NBT invalid"));
    }
    Ok(Some(named.tag))
}

// A bounded structural pass BEFORE SDK BTreeMap normalization. It does not decode
// saves or recreate Tags. SDK String decoding gives Java UTF-16 key equality,
// including accepted overlong modified-UTF8 encodings of the same key.
struct Audit<'a> {
    bytes: &'a [u8],
    offset: usize,
    nodes: usize,
}
impl<'a> Audit<'a> {
    fn take(&mut self, size: usize) -> GuardResult<&'a [u8]> {
        let end = self
            .offset
            .checked_add(size)
            .ok_or_else(|| fail("NBT length overflow"))?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| fail("truncated raw NBT"))?;
        self.offset = end;
        Ok(value)
    }
    fn byte(&mut self) -> GuardResult<u8> {
        Ok(self.take(1)?[0])
    }
    fn string_bytes(&mut self) -> GuardResult<&'a [u8]> {
        let len = self.take(2)?;
        self.take(u16::from_be_bytes([len[0], len[1]]) as usize)
    }
    fn key(&mut self) -> GuardResult<JavaString> {
        let raw = self.string_bytes()?;
        let mut wire = Vec::with_capacity(raw.len() + 3);
        wire.push(8);
        wire.extend_from_slice(&(raw.len() as u16).to_be_bytes());
        wire.extend_from_slice(raw);
        match nbt::read_network(&wire, limits()).map_err(|_| fail("invalid NBT key encoding"))? {
            (Tag::String(key), consumed) if consumed == wire.len() => Ok(key),
            _ => Err(fail("invalid NBT key")),
        }
    }
    fn count(&mut self) -> GuardResult<usize> {
        let b = self.take(4)?;
        let n = i32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        usize::try_from(n).map_err(|_| fail("negative NBT count"))
    }
    fn payload(&mut self, kind: u8, depth: usize) -> GuardResult<()> {
        self.nodes += 1;
        if self.nodes > MAX_NODES || depth > MAX_DEPTH {
            return Err(fail("raw NBT structural limit exceeded"));
        }
        match kind {
            1 => {
                self.take(1)?;
            }
            2 => {
                self.take(2)?;
            }
            3 | 5 => {
                self.take(4)?;
            }
            4 | 6 => {
                self.take(8)?;
            }
            7 | 11 | 12 => {
                let n = self.count()?;
                let width = match kind {
                    11 => 4,
                    12 => 8,
                    _ => 1,
                };
                self.take(
                    n.checked_mul(width)
                        .ok_or_else(|| fail("NBT array length overflow"))?,
                )?;
            }
            8 => {
                self.string_bytes()?;
            }
            9 => {
                let element = self.byte()?;
                let count = self.count()?;
                if count > MAX_NODES.saturating_sub(self.nodes)
                    || (count > 0 && !(1..=12).contains(&element))
                {
                    return Err(fail("invalid raw NBT list"));
                }
                for _ in 0..count {
                    self.payload(element, depth + 1)?;
                }
            }
            10 => {
                let mut keys = BTreeSet::new();
                loop {
                    let kind = self.byte()?;
                    if kind == 0 {
                        break;
                    }
                    if !keys.insert(self.key()?) {
                        return Err(fail("duplicate raw NBT compound key"));
                    }
                    self.payload(kind, depth + 1)?;
                }
            }
            _ => return Err(fail("invalid raw NBT tag type")),
        }
        Ok(())
    }
}
fn audit_named(bytes: &[u8]) -> GuardResult<()> {
    if bytes.len() > MAX_DAT_BYTES {
        return Err(fail("raw NBT byte limit exceeded"));
    }
    let mut audit = Audit {
        bytes,
        offset: 0,
        nodes: 0,
    };
    if audit.byte()? != 10 {
        return Err(fail("raw NBT root must be Compound"));
    }
    audit.string_bytes()?; // SDK deliberately ignores malformed file root names too.
    audit.payload(10, 0)?;
    if audit.offset != bytes.len() {
        return Err(fail("trailing raw NBT bytes"));
    }
    Ok(())
}
