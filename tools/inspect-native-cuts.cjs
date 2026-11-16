'use strict';
const fs = require('fs'), path = require('path'), zlib = require('zlib'), crypto = require('crypto'), assert = require('assert');
function readNbt(file) {
  const raw = fs.readFileSync(file), bytes = raw[0] === 31 && raw[1] === 139 ? zlib.gunzipSync(raw) : raw;
  let pos = 0;
  const take = n => { if (!Number.isSafeInteger(n) || n < 0 || pos + n > bytes.length) throw Error('NBT length'); const b = bytes.subarray(pos, pos + n); pos += n; return b; };
  const u8 = () => take(1)[0], i32 = () => take(4).readInt32BE(), str = () => new TextDecoder('utf-8', {fatal:true}).decode(take(take(2).readUInt16BE()));
  const count = () => { const n = i32(); if (n < 0 || n > 10000000) throw Error('NBT count'); return n; };
  function value(t, depth = 0) {
    if (depth > 512) throw Error('NBT depth');
    let v;
    switch (t) {
      case 1: v = take(1).readInt8(); break; case 2: v = take(2).readInt16BE(); break;
      case 3: v = i32(); break; case 4: v = take(8).readBigInt64BE().toString(); break;
      case 5: v = take(4).toString('hex'); break; case 6: v = take(8).toString('hex'); break;
      case 7: v = take(count()).toString('hex'); break; case 8: v = str(); break;
      case 9: { const child = u8(), n = count(); if (child > 12 || (child === 0 && n !== 0)) throw Error('NBT list type'); v = {child, items:Array.from({length:n}, () => value(child,depth+1))}; break; }
      case 10: { v = Object.create(null); for (;;) { const child = u8(); if (!child) break; const key = str(); if (Object.hasOwn(v,key)) throw Error('duplicate NBT'); v[key] = value(child,depth+1); } break; }
      case 11: v = Array.from({length:count()}, i32); break;
      case 12: v = Array.from({length:count()}, () => take(8).readBigInt64BE().toString()); break;
      default: throw Error('NBT type ' + t);
    }
    return {t,v};
  }
  const type = u8(); if (type !== 10) throw Error('NBT root type'); str();
  const root = value(type); if (pos !== bytes.length) throw Error('NBT trailing bytes');
  return {root, sha256:crypto.createHash('sha256').update(raw).digest('hex').toUpperCase()};
}
const stable = x => JSON.stringify(x, function(k,v) { if (v && !Array.isArray(v) && typeof v === 'object') return Object.fromEntries(Object.keys(v).sort().map(k => [k,v[k]])); return v; });
const clone = x => JSON.parse(JSON.stringify(x));
const eq = (a,b) => stable(a) === stable(b);
function extract(parsed, owner, worldId) {
  const root = parsed.root.v, ns = root.NeoForgeData?.v, state = ns?.['allomancy:state']?.v;
  if (!root.UUID || root.UUID.t !== 11 || root.UUID.v.length !== 4 || root.UUID.v.map(n => (n >>> 0).toString(16).padStart(8,'0')).join('') !== owner.replaceAll('-','')) throw Error('owner UUID');
  if (!root.Inventory || root.Inventory.t !== 9 || root.Inventory.v.child !== 10 || !state || state.format_version?.v !== 1 || state.payload?.t !== 7 || ns['allomancy:storage_fault']) throw Error('missing inventory, state, or storage fault');
  const envelope = JSON.parse(new TextDecoder('utf-8', {fatal:true}).decode(Buffer.from(state.payload.v,'hex'))), saved = envelope.saved_player;
  (function checkNumbers(x) { if (typeof x === 'number' && !Number.isSafeInteger(x)) throw Error('unsafe JSON number'); if (x && typeof x === 'object') Object.values(x).forEach(checkNumbers); })(envelope);
  if (envelope.schema_version !== 1 || !saved || saved.state?.player_id !== owner.replaceAll('-','') || saved.state.world_id !== worldId || !Array.isArray(saved.state.metals) || saved.state.metals.length !== 8 || !Array.isArray(saved.ledger) || !Number.isSafeInteger(saved.revision)) throw Error('payload identity or schema');
  const foreign = clone(ns); delete foreign['allomancy:state'];
  const stateMetadata = clone(state); delete stateMetadata.payload;
  const envelopeOther = clone(envelope); delete envelopeOther.saved_player;
  return {inventory:root.Inventory, saved, foreign, stateMetadata, envelopeOther};
}
function candidate(a, transaction, slot) {
  const b = clone(a), items = b.inventory.v.items;
  const matches = items.filter(x => x.v.Slot?.v === slot);
  if (matches.length !== 1 || matches[0].v.id?.v !== 'minecraft:iron_ingot' || !Number.isInteger(matches[0].v.count?.v) || matches[0].v.count.v < 1) throw Error('expected iron slot');
  const stack = matches[0];
  if (--stack.v.count.v === 0) items.splice(items.indexOf(stack),1);
  if (!Number.isSafeInteger(b.saved.state.metals[0].reserve_milli) || !Number.isSafeInteger(b.saved.state.metals[0].reserve_milli + 1000) || !Number.isSafeInteger(b.saved.revision + 1) || b.saved.ledger.some(x => x.transaction === transaction)) throw Error('candidate overflow or duplicate transaction');
  b.saved.state.metals[0].reserve_milli += 1000; b.saved.revision++;
  b.saved.ledger.push({transaction,payload:{metal:'Iron',amount_milli:1000},committed_revision:b.saved.revision});
  b.saved.ledger.sort((x,y) => x.transaction < y.transaction ? -1 : x.transaction > y.transaction ? 1 : 0);
  return b;
}
function reconnect(a) {
  const b = clone(a); if (!Number.isSafeInteger(b.saved.revision + 1)) throw Error('revision overflow');
  b.saved.revision++; b.saved.state.last_tick = null;
  for (const metal of b.saved.state.metals) metal.burn = 'Off';
  return b;
}
function classify(actual, variants) {
  const hit = variants.filter(([,v]) => eq(actual,v));
  if (!hit.length) throw Error('snapshot differs from every complete expected state');
  return hit.map(([label]) => label);
}
function allowedStates(mode, suffix, selected, old) {
  const current = [selected], reconnected = [['reconnected-'+selected[0],reconnect(selected[1])]];
  if (mode === 'cut') return suffix === '.dat_old' && old ? [...current,old] : current;
  if (suffix === '.dat') return reconnected;
  return old ? [...current,...reconnected,old] : [...current,...reconnected];
}
function main(args) {
  if (args.length === 1 && args[0] === '--self-test') {
    const a = {inventory:{v:{items:[{v:{Slot:{v:0},id:{v:'minecraft:iron_ingot'},count:{v:2}}}]}},saved:{state:{metals:Array.from({length:8},()=>({reserve_milli:1000,burn:'Off'})),last_tick:null},revision:5,ledger:[]},foreign:{x:{t:3,v:7}}};
    const b = candidate(a,'native-cut-test',0), variants = [['A',a],['B',b]];
    assert.deepStrictEqual(classify(a,variants),['A']); assert.deepStrictEqual(classify(b,variants),['B']);
    const mixed = clone(a); mixed.inventory = b.inventory; assert.throws(() => classify(mixed,variants));
    const corrupted = clone(b); corrupted.foreign.x.v++; assert.throws(() => classify(corrupted,variants));
    const selected = ['A',a];
    assert.deepStrictEqual(classify(a,allowedStates('cut','.dat',selected,null)),['A']);
    assert.throws(() => classify(b,allowedStates('cut','.dat',selected,null)));
    assert.deepStrictEqual(classify(reconnect(a),allowedStates('reopen','.dat',selected,null)),['reconnected-A']);
    assert.throws(() => classify(reconnect(b),allowedStates('reopen','.dat',selected,null)));
    assert.throws(() => classify(b,allowedStates('reopen','.dat_new',selected,null)));
    const os = require('os'), dir = fs.mkdtempSync(path.join(os.tmpdir(),'native-cuts-')), file = path.join(dir,'fixture.dat');
    try {
      fs.writeFileSync(file,Buffer.from([10,0,0,1,0,1,97,1,1,0,1,97,2,0]));
      assert.throws(() => readNbt(file), /duplicate NBT/);
      fs.writeFileSync(file,Buffer.from([10,0,0,0,1]));
      assert.throws(() => readNbt(file), /trailing/);
    } finally { fs.rmSync(dir,{recursive:true,force:true}); }
    console.log('self-test PASS'); return;
  }
  if (args.length % 2) throw Error('arguments require values');
  const o = {}; for (let i=0;i<args.length;i+=2) { if (!args[i].startsWith('--') || Object.hasOwn(o,args[i])) throw Error('invalid or duplicate argument'); o[args[i]]=args[i+1]; }
  const required = ['--baseline','--world','--owner','--world-id','--mode','--transaction','--slot','--expect'];
  if (Object.keys(o).some(k => ![...required,'--baseline-old'].includes(k)) || required.some(k => !o[k]) || !/^[0-9a-f]{8}-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}$/.test(o['--owner']) || !/^[0-9a-f]{32}$/.test(o['--world-id']) || !/^native-cut-[A-Za-z0-9_-]{1,64}$/.test(o['--transaction']) || !/^(0|[1-9][0-9]*)$/.test(o['--slot']) || !['cut','reopen'].includes(o['--mode']) || !['A','B'].includes(o['--expect'])) throw Error('invalid arguments');
  const owner=o['--owner'], worldId=o['--world-id'], baseline=readNbt(o['--baseline']), a=extract(baseline,owner,worldId), b=candidate(a,o['--transaction'],Number(o['--slot']));
  const selected=o['--expect']==='A' ? ['A',a] : ['B',b];
  let oldHash = null, oldVariant = null;
  if (o['--baseline-old']) { const old=readNbt(o['--baseline-old']); oldHash=old.sha256; const oldState=extract(old,owner,worldId); if (!eq(oldState,a) && !eq(reconnect(oldState),a)) throw Error('baseline old is not A or its pre-reconnect predecessor'); oldVariant=['baseline-old',oldState]; }

  const dir=path.join(o['--world'],'players','data'), snapshots=[];
  for (const suffix of ['.dat','.dat_old','.dat_new']) {
    const file=path.join(dir,owner+suffix);
    if (!fs.existsSync(file)) { if (suffix === '.dat') throw Error('primary missing'); snapshots.push({file,present:false}); continue; }
    const parsed=readNbt(file), actual=extract(parsed,owner,worldId);
    const allowed=allowedStates(o['--mode'],suffix,selected,oldVariant);
    snapshots.push({file,present:true,sha256:parsed.sha256,matches:classify(actual,allowed)});
  }
  console.log(JSON.stringify({oracle:'PASS',mode:o['--mode'],expect:o['--expect'],baseline:{file:o['--baseline'],sha256:baseline.sha256,old_sha256:oldHash},transaction:o['--transaction'],slot:Number(o['--slot']),snapshots},null,2));
}
try { main(process.argv.slice(2)); } catch (e) { console.error('oracle FAIL: '+e.message); process.exitCode=1; }
