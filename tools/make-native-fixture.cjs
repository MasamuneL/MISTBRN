'use strict';
// Offline developer fixtures only. Caller must ensure the world and game are closed.
const fs = require('node:fs');
const path = require('node:path');
const zlib = require('node:zlib');
const crypto = require('node:crypto');

const kinds = ['foreign', 'fault', 'fault-scalar', 'fault-future', 'fault-without-state',
  'future-transport', 'future-json', 'corrupt-json', 'invalid-root', 'duplicate',
  'truncated', 'dat-uppercase', 'backup-fault'];
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
} catch (error) {
  process.stderr.write('fixture error: ' + error.message + '\n');
  process.exitCode = 1;
}
