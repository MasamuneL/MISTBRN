'use strict';
const fs = require('fs'), path = require('path'), zlib = require('zlib'), crypto = require('crypto'), assert = require('assert');
function readNbt(file) {
  const raw = fs.readFileSync(file), bytes = raw[0] === 31 && raw[1] === 139 ? zlib.gunzipSync(raw, {maxOutputLength:32*1024*1024}) : raw;
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
  if (!root.Inventory || root.Inventory.t !== 9 || !(root.Inventory.v.child === 10 || (root.Inventory.v.child === 0 && root.Inventory.v.items.length === 0)) || root.NeoForgeData?.t !== 10 || ns?.['allomancy:state']?.t !== 10 || !state || state.format_version?.t !== 3 || state.format_version?.v !== 1 || state.payload?.t !== 7 || ns['allomancy:storage_fault']) throw Error('missing inventory, state, or storage fault');
  const envelope = JSON.parse(new TextDecoder('utf-8', {fatal:true}).decode(Buffer.from(state.payload.v,'hex'))), saved = envelope.saved_player;
  (function checkNumbers(x) { if (typeof x === 'number' && !Number.isSafeInteger(x)) throw Error('unsafe JSON number'); if (x && typeof x === 'object') Object.values(x).forEach(checkNumbers); })(envelope);
  if (envelope.schema_version !== 1 || !saved || saved.state?.player_id !== owner.replaceAll('-','') || saved.state.world_id !== worldId || !Array.isArray(saved.state.metals) || saved.state.metals.length !== 8 || !Array.isArray(saved.ledger) || !Number.isSafeInteger(saved.revision)) throw Error('payload identity or schema');
  const foreign = clone(ns); delete foreign['allomancy:state'];
  const stateMetadata = clone(state); delete stateMetadata.payload;
  const envelopeOther = clone(envelope); delete envelopeOther.saved_player;
  return {inventory:root.Inventory, saved, foreign, stateMetadata, envelopeOther};
}
function expected(a,mode) {
 const b=clone(a); if(mode==='unchanged'||mode==='identity')return b;
 let delta=mode==='death-login'?2:1;
 if(!Number.isSafeInteger(b.saved.revision+delta))throw Error('revision overflow');
 b.saved.revision+=delta;b.saved.state.last_tick=null;
 for(const metal of b.saved.state.metals){metal.burn='Off';if(mode.startsWith('death-'))metal.reserve_milli=0}
 if(mode.startsWith('death-')) b.saved.accounting.remainder=Array(8).fill(0);
 if(mode==='death-respawn'){b.inventory.v.items=[];b.inventory.v.child=0;}
 return b;
}
function verify(actual,expectedValue){if(!eq(actual,expectedValue))throw Error('complete state differs from expected');}
function creditExpected(base) {
 const target=expected(base,'reconnect'), revision=target.saved.revision;
 assert.equal(base.inventory.v.items.length,1);const item=target.inventory.v.items[0].v;
 assert.equal(item.Slot.v,0);assert.equal(item.id.v,'minecraft:iron_ingot');assert.equal(item.count.v,2);item.count.v=1;
 target.saved.revision++;target.saved.state.metals[0].reserve_milli+=1000;
 target.saved.ledger.push({transaction:`native-roundtrip-${revision}`,payload:{metal:'Iron',amount_milli:1000},committed_revision:revision+1});return target;
}
if(process.argv[2]==='--self-test') {
 const base={inventory:{v:{items:[{v:{Slot:{v:0},id:{v:'minecraft:iron_ingot'},count:{v:2}}}]}},saved:{revision:4,state:{last_tick:99,metals:Array.from({length:8},()=>({reserve_milli:0,burn:'Off'}))},ledger:[]},foreign:{retained:7}};
 const target=creditExpected(base);verify(target,creditExpected(base));
 for(const mutate of [x=>x.inventory.v.items[0].v.count.v++,x=>x.saved.ledger.pop(),x=>x.saved.state.metals[0].reserve_milli++,x=>x.foreign.retained++]) {const bad=clone(target);mutate(bad);assert.throws(()=>verify(bad,creditExpected(base)));}console.log('self-test PASS');
} else {
const runtime=process.argv[2]; if(!runtime||process.argv.length!==3)throw Error('runtime argument required');
const owner='3efd2c0e-f0de-3e85-b045-dec265b3f79c', wid='026e465a87434711a6951c07588dc0ee';
const hash=f=>crypto.createHash('sha256').update(fs.readFileSync(f)).digest('hex').toUpperCase();
function checkManifest(dir, entries) {assert(Array.isArray(entries)&&entries.length>0);const actual=[];function walk(d){for(const e of fs.readdirSync(d,{withFileTypes:true})){assert(!e.isSymbolicLink());const f=path.join(d,e.name);if(e.isDirectory())walk(f);else actual.push({path:path.relative(dir,f).replaceAll('/','\\'),bytes:fs.statSync(f).size,sha256:hash(f)});}}walk(dir);assert.equal(actual.length,entries.length);for(const e of entries)assert(eq(actual.find(x=>x.path===e.path),e),'manifest changed: '+e.path);}
const results=[];
for(const name of ['gates-replay-new-1','gates-reentrant-1']) {
 const dir=path.join(runtime,'native-gate-credit-runs',name), receipt=path.join(dir,'result.json'),r=JSON.parse(fs.readFileSync(receipt));
 assert.equal(r.scenario,name==='gates-replay-new-1'?'replay-new':'reentrant');assert.equal(r.native_pass,true);assert.equal(r.exit_code,0);assert.equal(r.termination,'spontaneous_after_quit');assert.equal(r.error,null);
 checkManifest(r.baseline,r.baseline_files);checkManifest(r.backup,r.backup_files);checkManifest(r.world,r.after_files);
 const stderr=path.join(dir,'stderr.log'),stdout=path.join(dir,'stdout.log'), log=fs.readFileSync(stderr,'utf8')+fs.readFileSync(stdout,'utf8');
 assert(log.includes(`ALLOMANCY_NATIVE_TEST scenario=${r.scenario} result=PASS durable=false`));assert(log.includes('phase=WorldCloseFinal'));assert(!/result=FAIL|event=(fault|world_fault|quarantine_confirmed|quarantine_unconfirmed|rollback_unverified)/.test(log));
 assert(log.includes(r.scenario==='replay-new'?'event=replay_unchanged scenario=replay-new unchanged=true':'event=reentrant_rejected inner=Busy outer=Busy induced_adapter_recursion=true unchanged=true'));
 const file=d=>path.join(d,'players/data',owner+'.dat'),baseFile=file(r.baseline),base=extract(readNbt(baseFile),owner,wid),target=creditExpected(base),snapshots=[];
 assert.equal(base.saved.revision,4);assert.equal(base.saved.state.metals[0].reserve_milli,1000);assert.equal(base.saved.ledger.length,1);
 for(const suffix of ['', '_old']) {const f=file(r.world)+suffix,p=readNbt(f);verify(extract(p,owner,wid),target);snapshots.push({file:f,sha256:p.sha256});}
 assert(!fs.existsSync(file(r.world)+'_new'));
 assert.equal(fs.readFileSync(path.join(r.world,'allomancy/world-identity.json'),'utf8'),fs.readFileSync(path.join(r.baseline,'allomancy/world-identity.json'),'utf8'));
 results.push({name,oracle:'PASS',receipt:{path:receipt,sha256:hash(receipt)},logs:[stderr,stdout].map(f=>({path:f,sha256:hash(f)})),baseline:{path:baseFile,sha256:hash(baseFile)},snapshots,expected:target,native_scope:r.scenario==='reentrant'?'induced adapter recursion Busy; subsequent single credit':'persisted replay unchanged; subsequent single new credit'});
}
console.log(JSON.stringify({oracle:'PASS',scope:'offline owner integrated N3/N4; no durability or global P2 claim',inspector_sha256:hash(__filename),results},null,2));
}