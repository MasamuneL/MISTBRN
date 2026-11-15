use allomancy_core::{
    LoadedWorldIdentity, MAX_WORLD_IDENTITY_BYTES, WorldId, WorldIdentityError as E,
    WorldIdentitySource as S, decode_world_identity, load_world_identity,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

const ID: &str = "00112233445566778899aabbccddeeff";
fn marker(id: &str) -> Vec<u8> {
    format!(r#"{{"schema_version":1,"world_id":"{id}"}}"#).into_bytes()
}
struct World(PathBuf);
impl World {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "allomancy-identity-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::create_dir(path.join("allomancy")).unwrap();
        Self(path)
    }
    fn primary(&self) -> PathBuf {
        self.0.join("allomancy/world-identity.json")
    }
    fn backup(&self) -> PathBuf {
        self.0.join("allomancy/world-identity.json.bak")
    }
    fn load(&self) -> Result<Option<LoadedWorldIdentity>, E> {
        load_world_identity(&self.0)
    }
}
impl Drop for World {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn strict_marker_returns_uuid_without_normalization() {
    assert_eq!(decode_world_identity(&marker(ID)), Ok(WorldId::new(ID)));
    assert_eq!(
        decode_world_identity(
            format!(" \n{} \t", String::from_utf8(marker(ID)).unwrap()).as_bytes()
        ),
        Ok(WorldId::new(ID))
    );
}
#[test]
fn id_requires_exactly_32_lowercase_hex_digits() {
    for id in [
        "",
        "A0112233445566778899aabbccddeeff",
        "00112233-4455-6677-8899-aabbccddeeff",
        "g0112233445566778899aabbccddeeff",
        "00112233445566778899aabbccddeef",
        "00112233445566778899aabbccddeeff0",
    ] {
        assert_eq!(
            decode_world_identity(&marker(id)),
            Err(E::InvalidId),
            "{id}"
        );
    }
}
#[test]
fn exact_object_rejects_missing_extra_wrong_types_arrays_and_null() {
    for bytes in [
        r#"{}"#,
        r#"{"schema_version":1}"#,
        r#"{"world_id":"00112233445566778899aabbccddeeff"}"#,
        r#"{"schema_version":1,"world_id":null}"#,
        r#"{"schema_version":1,"world_id":32}"#,
        r#"{"schema_version":"1","world_id":"00112233445566778899aabbccddeeff"}"#,
        r#"{"schema_version":0,"world_id":"00112233445566778899aabbccddeeff"}"#,
        r#"{"schema_version":1,"world_id":"00112233445566778899aabbccddeeff","extra":true}"#,
        r#"[1,"00112233445566778899aabbccddeeff"]"#,
        "null",
    ] {
        assert_eq!(
            decode_world_identity(bytes.as_bytes()),
            Err(E::Schema),
            "{bytes}"
        );
    }
}
#[test]
fn duplicate_fields_reject_even_identical_values_or_escaped_names() {
    for bytes in [
        r#"{"schema_version":1,"schema_version":1,"world_id":"00112233445566778899aabbccddeeff"}"#,
        r#"{"schema_version":1,"world_id":"00112233445566778899aabbccddeeff","world_id":"00112233445566778899aabbccddeeff"}"#,
        r#"{"schema_version":1,"world_id":"00112233445566778899aabbccddeeff","world_\u0069d":"00112233445566778899aabbccddeeff"}"#,
    ] {
        assert_eq!(decode_world_identity(bytes.as_bytes()), Err(E::Schema));
    }
}
#[test]
fn future_version_is_reported_before_v1_payload_shape() {
    assert_eq!(
        decode_world_identity(br#"{"schema_version":2,"world_id":null,"future_payload":[]}"#),
        Err(E::Future(2))
    );
}
#[test]
fn invalid_utf8_json_truncation_and_trailing_data_reject() {
    for bytes in [
        vec![0xff],
        b"{".to_vec(),
        [marker(ID), b" x".to_vec()].concat(),
    ] {
        assert_eq!(decode_world_identity(&bytes), Err(E::Parse));
    }
}
#[test]
fn size_limit_is_enforced_before_parse_and_allows_exact_boundary() {
    let mut bytes = marker(ID);
    bytes.resize(MAX_WORLD_IDENTITY_BYTES, b' ');
    assert_eq!(decode_world_identity(&bytes), Ok(WorldId::new(ID)));
    bytes.push(b' ');
    assert_eq!(decode_world_identity(&bytes), Err(E::TooLarge));
}
#[test]
fn absent_pair_returns_none_and_does_not_create_files() {
    let world = World::new();
    assert_eq!(world.load(), Ok(None));
    assert_eq!(fs::read_dir(world.0.join("allomancy")).unwrap().count(), 0);
}
#[test]
fn absent_world_returns_none_without_creating_root() {
    let world = World::new();
    let absent = world.0.join("absent");
    assert_eq!(load_world_identity(&absent), Ok(None));
    assert!(!absent.exists());
}
#[test]
fn primary_alone_returns_primary_and_does_not_create_backup() {
    let world = World::new();
    fs::write(world.primary(), marker(ID)).unwrap();
    assert_eq!(
        world.load(),
        Ok(Some(LoadedWorldIdentity {
            world_id: WorldId::new(ID),
            source: S::Primary
        }))
    );
    assert!(!world.backup().exists());
}
#[test]
fn missing_primary_valid_backup_is_explicit_and_never_rewritten() {
    let world = World::new();
    let bytes = marker(ID);
    fs::write(world.backup(), &bytes).unwrap();
    assert_eq!(
        world.load(),
        Ok(Some(LoadedWorldIdentity {
            world_id: WorldId::new(ID),
            source: S::Backup
        }))
    );
    assert!(!world.primary().exists());
    assert_eq!(fs::read(world.backup()).unwrap(), bytes);
}
#[test]
fn semantically_equal_pair_accepts_different_whitespace_and_order() {
    let world = World::new();
    fs::write(world.primary(), marker(ID)).unwrap();
    fs::write(
        world.backup(),
        format!(r#"{{ "world_id":"{ID}", "schema_version":1 }}"#),
    )
    .unwrap();
    assert_eq!(world.load().unwrap().unwrap().source, S::Primary);
}
#[test]
fn valid_but_conflicting_pair_rejects_without_mutation() {
    let world = World::new();
    let a = marker(ID);
    let b = marker("11112233445566778899aabbccddeeff");
    fs::write(world.primary(), &a).unwrap();
    fs::write(world.backup(), &b).unwrap();
    assert_eq!(world.load(), Err(E::Conflict));
    assert_eq!(fs::read(world.primary()).unwrap(), a);
    assert_eq!(fs::read(world.backup()).unwrap(), b);
}
#[test]
fn invalid_primary_never_falls_back_to_valid_backup() {
    let world = World::new();
    for (bytes, error) in [
        (b"{".to_vec(), E::Parse),
        (br#"{"schema_version":3}"#.to_vec(), E::Future(3)),
        (vec![b' '; MAX_WORLD_IDENTITY_BYTES + 1], E::TooLarge),
    ] {
        fs::write(world.primary(), &bytes).unwrap();
        fs::write(world.backup(), marker(ID)).unwrap();
        assert_eq!(world.load(), Err(error));
        assert_eq!(fs::read(world.primary()).unwrap(), bytes);
    }
}
#[test]
fn invalid_backup_is_not_ignored_when_primary_is_valid() {
    let world = World::new();
    fs::write(world.primary(), marker(ID)).unwrap();
    fs::write(world.backup(), b"{").unwrap();
    assert_eq!(world.load(), Err(E::Parse));
}
#[test]
fn directory_in_either_file_position_is_io_error() {
    for backup in [false, true] {
        let world = World::new();
        fs::create_dir(if backup {
            world.backup()
        } else {
            world.primary()
        })
        .unwrap();
        assert_eq!(world.load(), Err(E::Io));
    }
}
