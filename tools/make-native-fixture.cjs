'use strict';
// Offline developer fixtures only. Caller must ensure the world and game are closed.
const fs = require('node:fs');
const path = require('node:path');
const zlib = require('node:zlib');
const crypto = require('node:crypto');

const kinds = ['foreign', 'fault', 'fault-scalar', 'fault-future', 'fault-without-state',
  'future-transport', 'future-json', 'corrupt-json', 'invalid-root', 'duplicate',
  'truncated', 'dat-uppercase', 'backup-fault', 'level-owner-missing', 'player-foreign'];
function fail(message) { throw new Error(message); }
function u16(n) { const b = Buffer.alloc(2); b.writeUInt16BE(n); return b; }
function i32(n) { const b = Buffer.alloc(4); b.writeInt32BE(n); return b; }
function name(s) { const b = Buffer.from(s, 'utf8'); return Buffer.concat([u16(b.length), b]); }
function tag(type, key, value) { return Buffer.concat([Buffer.from([type]), name(key), value]); }
function int(key, value) { return tag(3, key, i32(value)); }
function byte(key, value) { return tag(1, key, Buffer.from([value])); }
function str(key, value) { return tag(8, key, name(value)); }
function compound(key, entries) { return tag(10, key, Buffer.concat([...entries, Buffer.from([0])])); }
function bytes(key, value) { return tag(7, key, Buffer.concat([i32(value.length), value])); }
function fault(version = 1) {
  return compound('allomancy:storage_fault', [int('format_version', version), str('reason', 'rollback_unverified')]);
}
function state(version, payload) {
  return compound('allomancy:state', [int('format_version', version), bytes('payload', Buffer.from(payload))]);
}
function player(kind) {
  const uuid = tag(11, 'UUID', Buffer.concat([i32(4), i32(0), i32(0), i32(0), i32(1)]));
  const health = Buffer.alloc(4); health.writeFloatBE(20);
  let mods;
  switch (kind) {
    case 'foreign': case 'backup-fault': mods = compound('NeoForgeData', [int('foreign:keep', 7)]); break;
    case 'fault': case 'fault-without-state': mods = compound('NeoForgeData', [fault()]); break;
    case 'fault-scalar': mods = compound('NeoForgeData', [byte('allomancy:storage_fault', 0)]); break;
    case 'fault-future': mods = compound('NeoForgeData', [fault(2)]); break;
    case 'future-transport': mods = compound('NeoForgeData', [state(2, '')]); break;
    case 'future-json': mods = compound('NeoForgeData', [state(1, '{"schema_version":999}')]); break;
    case 'corrupt-json': mods = compound('NeoForgeData', [state(1, '{"schema_version":')]); break;
    case 'invalid-root': mods = int('NeoForgeData', 7); break;
    default: mods = compound('NeoForgeData', [int('foreign:keep', 7)]);
  }
  const duplicate = kind === 'duplicate' || kind === 'dat-uppercase'
    ? [int('duplicate', 1), int('duplicate', 2)] : [];
  const raw = Buffer.concat([Buffer.from([10, 0, 0]), uuid, tag(5, 'Health', health), mods, ...duplicate, Buffer.from([0])]);
  const gzip = zlib.gzipSync(raw);
  return kind === 'truncated' ? gzip.subarray(0, 10) : gzip;
}
function readNbt(file){
 const original=fs.readFileSync(file);
 const bytes=original[0]===31&&original[1]===139?zlib.gunzipSync(original):original;
 let at=0, modsEnd=null;
 const take=n=>{if(n<0||at+n>bytes.length)throw Error('NBT length');const b=bytes.subarray(at,at+n);at+=n;return b;};
 const u8=()=>take(1)[0],i32=()=>take(4).readInt32BE(),str=()=>take(take(2).readUInt16BE()).toString('utf8');
 const count=()=>{const n=i32();if(n<0||n>10000000)throw Error('NBT count');return n;};
 function value(type,depth=0,route=[]){
 if(depth>512)throw Error('NBT depth');
 switch(type){
 case 1:return take(1).readInt8();case 2:return take(2).readInt16BE();case 3:return i32();case 4:return take(8).readBigInt64BE();case 5:return take(4).readFloatBE();case 6:return take(8).readDoubleBE();case 7:return take(count());case 8:return str();
 case 9:{const child=u8(),n=count();return Array.from({length:n},()=>value(child,depth+1,[...route,'[]']));}
 case 10:{const out=Object.create(null);while(true){const end=at,child=u8();if(!child){if(route.length===1&&route[0]==='NeoForgeData')modsEnd=end;return out;}const key=str();if(Object.hasOwn(out,key))throw Error('duplicate NBT');out[key]=value(child,depth+1,[...route,key]);}}
 case 11:return Array.from({length:count()},i32);case 12:return Array.from({length:count()},()=>take(8).readBigInt64BE());default:throw Error('NBT type '+type);
 }}
 const type=u8();str();const result=value(type);if(at!==bytes.length)throw Error('NBT trailing bytes');
 return {root:result,raw:bytes,original,modsEnd,hash:crypto.createHash('sha256').update(original).digest('hex').toUpperCase()};
}
const uuid=ints=>Array.isArray(ints)&&ints.length===4?ints.map(n=>(n>>>0).toString(16).padStart(8,'0')).join(''):null;
function playerForeign(world) {
  const owner = uuid(readNbt(path.join(world, 'level.dat')).root.Data?.singleplayer_uuid);
  if (!owner) fail('level owner UUID required');
  const stem = owner.replace(/^(.{8})(.{4})(.{4})(.{4})(.{12})$/, '$1-$2-$3-$4-$5');
  const data = path.join(world, 'players', 'data');
  for (const dir of [path.dirname(data), data]) {
    const st = fs.lstatSync(dir);
    if (!st.isDirectory() || st.isSymbolicLink()) fail('player directory must be real');
  }
  if (fs.realpathSync(data) !== data) fail('player directory redirects');
  const snapshots = [stem + '.dat', stem + '.dat_old'].map(file => {
    const target = path.join(data, file), stat = fs.lstatSync(target);
    if (!stat.isFile() || stat.isSymbolicLink()) fail('player snapshot must be regular');
    const nbt = readNbt(target), mods = nbt.root.NeoForgeData;
    const payload = mods?.['allomancy:state']?.payload;
    if (uuid(nbt.root.UUID) !== owner || nbt.modsEnd === null || !Buffer.isBuffer(payload)
        || mods['allomancy:state'].format_version !== 1 || Object.hasOwn(mods, 'foreign:keep')
        || Object.hasOwn(mods, 'allomancy:storage_fault')) fail('owner namespace unsuitable');
    const saved = JSON.parse(payload.toString('utf8'));
    if (saved.schema_version !== 1 || saved.saved_player?.state?.player_id !== owner) fail('payload identity mismatch');
    const edited = zlib.gzipSync(Buffer.concat([nbt.raw.subarray(0, nbt.modsEnd),
      int('foreign:keep', 7), nbt.raw.subarray(nbt.modsEnd)]));
    return {target, file, nbt, edited, payload};
  });
  const originals = path.join(world, '.fixture-original');
  fs.mkdirSync(originals);
  for (const item of snapshots) fs.writeFileSync(path.join(originals, item.file), item.nbt.original, {flag:'wx'});
  const hash = value => crypto.createHash('sha256').update(value).digest('hex');
  for (const item of snapshots) {
    fs.writeFileSync(item.target, item.edited);
    const actual = readNbt(item.target).root.NeoForgeData;
    if (actual['foreign:keep'] !== 7 || !actual['allomancy:state'].payload.equals(item.payload)) fail('fixture verification failed');
  }
  return snapshots.map(item => ({path:item.target, player:owner, foreign_keep:7,
    before_sha256:hash(item.nbt.original), after_sha256:hash(item.edited)}));
}
function levelOwnerMissing(world) {
  const paths = ['level.dat', 'level.dat_old'].map(n => path.join(world, n));
  const signature = Buffer.concat([Buffer.from([11]), name('singleplayer_uuid'), i32(4)]);
  const snapshots = paths.map(file => {
    const stat = fs.lstatSync(file);
    if (!stat.isFile() || stat.isSymbolicLink()) fail('level must be a regular file');
    const original = fs.readFileSync(file);
    if (original[0] !== 31 || original[1] !== 139) fail('level must be gzip');
    const parsed = readNbt(file).root;
    const data = parsed.Data;
    if (!data || typeof data !== 'object' || Object.hasOwn(data, 'Player') ||
        !Array.isArray(data.singleplayer_uuid) || data.singleplayer_uuid.length !== 4) {
      fail('level requires Data.singleplayer_uuid and no legacy Data.Player');
    }
    const raw = zlib.gunzipSync(original);
    const index = raw.indexOf(signature);
    if (index < 0 || raw.indexOf(signature, index + 1) !== -1 || index + signature.length + 16 > raw.length) {
      fail('owner tag signature absent, duplicated or truncated');
    }
    const encoded = Buffer.concat(data.singleplayer_uuid.map(i32));
    if (!raw.subarray(index + signature.length, index + signature.length + 16).equals(encoded)) {
      fail('owner signature does not match Data owner');
    }
    const edited = zlib.gzipSync(Buffer.concat([raw.subarray(0, index), raw.subarray(index + signature.length + 16)]));
    return {file, original, edited, owner: uuid(data.singleplayer_uuid)};
  });
  if (snapshots[0].owner !== snapshots[1].owner) fail('level owners differ');
  const id = snapshots[0].owner.replace(/^(.{8})(.{4})(.{4})(.{4})(.{12})$/, '$1-$2-$3-$4-$5');
  const players = path.join(world, 'players');
  const data = path.join(players, 'data');
  for (const dir of [players, data]) {
    const st = fs.lstatSync(dir);
    if (!st.isDirectory() || st.isSymbolicLink()) fail('player path must be a real directory');
  }
  if (path.dirname(fs.realpathSync(players)) !== world || path.dirname(fs.realpathSync(data)) !== fs.realpathSync(players)) {
    fail('player path escapes world');
  }
  const ownerFile = path.join(data, id + '.dat');
  const owner = fs.lstatSync(ownerFile);
  if (!owner.isFile() || owner.isSymbolicLink()) fail('owner player primary missing');
  const ownerRoot = readNbt(ownerFile).root;
  const ownerState = ownerRoot.NeoForgeData?.['allomancy:state'];
  if (uuid(ownerRoot.UUID) !== snapshots[0].owner || ownerState?.format_version !== 1 ||
      !Buffer.isBuffer(ownerState?.payload)) fail('owner player identity or state invalid');
  const payload = JSON.parse(ownerState.payload.toString('utf8'));
  if (payload?.saved_player?.state?.player_id !== snapshots[0].owner) fail('owner payload identity mismatch');
  const backups = path.join(world, '.fixture-original');
  fs.mkdirSync(backups);
  for (const item of snapshots) {
    fs.writeFileSync(path.join(backups, path.basename(item.file)), item.original, {flag:'wx'});
  }
  const hash = data => crypto.createHash('sha256').update(data).digest('hex');
  for (const item of snapshots) fs.writeFileSync(item.file, item.edited);
  return snapshots.map(item => ({path:item.file, before_sha256:hash(item.original), after_sha256:hash(item.edited)}));
}
try {
  const args = process.argv.slice(2);
  if (args.length !== 4 || args[0] !== '--world' || args[2] !== '--kind' || !kinds.includes(args[3])) {
    fail('Usage: node make-native-fixture.cjs --world <world> --kind <supported-kind>');
  }
  const kind = args[3];
  const world = fs.realpathSync(args[1]);
  if (!fs.statSync(world).isDirectory()) fail('world is not a directory');
  const saves = path.dirname(world);
  const game = path.dirname(saves);
  const runtime = path.dirname(game);
  if (path.basename(saves).toLowerCase() !== 'saves' ||
      !/^game-p2-.+$/i.test(path.basename(game)) ||
      path.basename(runtime).toLowerCase() !== 'runtime') {
    fail('world must be runtime/game-p2-*/saves/NAME');
  }
  if (!fs.statSync(path.join(world, 'allomancy', 'world-identity.json')).isFile()) fail('marker missing');
  if (kind === 'player-foreign') {
    const files = playerForeign(world);
    process.stdout.write(JSON.stringify({kind, expected:'namespace preserved by active-player operations', files}) + '\n');
  } else if (kind === 'level-owner-missing') {
    const files = levelOwnerMissing(world);
    process.stdout.write(JSON.stringify({kind, expected:'reject', files, player_files_modified:false}) + '\n');
  } else {
  const players = path.join(world, 'players');
  const data = path.join(players, 'data');
  for (const dir of [players, data]) {
    if (fs.existsSync(dir)) {
      const stat = fs.lstatSync(dir);
      if (stat.isSymbolicLink() || !stat.isDirectory()) fail('player path must be a real directory');
    } else {
      fs.mkdirSync(dir);
    }
  }
  if (path.dirname(fs.realpathSync(players)) !== world || path.dirname(fs.realpathSync(data)) !== fs.realpathSync(players)) {
    fail('player path escapes world');
  }
  const stem = '00000000-0000-0000-0000-000000000001';
  const names = fs.readdirSync(data).map(n => n.toLowerCase());
  if (names.includes(stem + '.dat') || names.includes(stem + '.dat_old')) fail('fixture player already exists');
  const files = [{ name: stem + (kind === 'dat-uppercase' ? '.DAT' : '.dat'), data: player(kind) }];
  if (kind === 'backup-fault') files.push({ name: stem + '.dat_old', data: player('fault') });
  const created = [];
  for (const file of files) {
    const target = path.join(data, file.name);
    fs.writeFileSync(target, file.data, { flag: 'wx' });
    created.push({ path: target, sha256: crypto.createHash('sha256').update(file.data).digest('hex'), bytes: file.data.length });
  }
  process.stdout.write(JSON.stringify({ kind, expected: kind === 'foreign' ? 'accept' : 'reject', files: created }) + '\n');
  }
} catch (error) {
  process.stderr.write('fixture error: ' + error.message + '\n');
  process.exitCode = 1;
}
