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
  if (!root.Inventory || root.Inventory.t !== 9 || !(root.Inventory.v.child === 10 || (root.Inventory.v.child === 0 && root.Inventory.v.items.length === 0)) || !state || state.format_version?.v !== 1 || state.payload?.t !== 7 || ns['allomancy:storage_fault']) throw Error('missing inventory, state, or storage fault');
  const envelope = JSON.parse(new TextDecoder('utf-8', {fatal:true}).decode(Buffer.from(state.payload.v,'hex'))), saved = envelope.saved_player;
  (function checkNumbers(x) { if (typeof x === 'number' && !Number.isSafeInteger(x)) throw Error('unsafe JSON number'); if (x && typeof x === 'object') Object.values(x).forEach(checkNumbers); })(envelope);
  if (envelope.schema_version !== 1 || !saved || saved.state?.player_id !== owner.replaceAll('-','') || saved.state.world_id !== worldId || !Array.isArray(saved.state.metals) || saved.state.metals.length !== 8 || !Array.isArray(saved.ledger) || !Number.isSafeInteger(saved.revision)) throw Error('payload identity or schema');
  const foreign = clone(ns); delete foreign['allomancy:state'];
  const stateMetadata = clone(state); delete stateMetadata.payload;
  const envelopeOther = clone(envelope); delete envelopeOther.saved_player;
  return {inventory:root.Inventory, saved, foreign, stateMetadata, envelopeOther};
}

function candidate(a) {
  if (a.inventory.v.items.length !== 0) throw Error('baseline inventory must be empty before give3');
  const b=clone(a), transaction='native-roundtrip-'+a.saved.revision;
  if (!Number.isSafeInteger(b.saved.revision+1) || !Number.isSafeInteger(b.saved.state.metals[0].reserve_milli+1000) || b.saved.ledger.some(e=>e.transaction===transaction)) throw Error('overflow or duplicate transaction');
  b.inventory={t:9,v:{child:10,items:[{t:10,v:{Slot:{t:1,v:0},id:{t:8,v:'minecraft:iron_ingot'},count:{t:3,v:2}}}]}};
  b.saved.state.metals[0].reserve_milli+=1000; b.saved.revision++;
  b.saved.ledger.push({transaction,payload:{metal:'Iron',amount_milli:1000},committed_revision:b.saved.revision});
  b.saved.ledger.sort((x,y)=>x.transaction<y.transaction?-1:x.transaction>y.transaction?1:0);
  return b;
}
function reconnect(a) {
  const b=clone(a); if (!Number.isSafeInteger(b.saved.revision+1)) throw Error('revision overflow');
  b.saved.revision++; b.saved.state.last_tick=null;
  b.saved.state.metals.forEach(m=>{m.burn='Off';}); return b;
}
function requireEqual(actual, expected) {
  if (!eq(actual,expected)) throw Error('complete inventory / SavedPlayer / namespace / envelope mismatch');
}
function selftest() {
  const a={inventory:{t:9,v:{child:0,items:[]}},saved:{revision:7,ledger:[],state:{last_tick:null,metals:Array.from({length:8},()=>({reserve_milli:0,burn:'Off'}))}},foreign:{x:{t:3,v:5}},stateMetadata:{format_version:{t:3,v:1}},envelopeOther:{schema_version:1}};
  const b=candidate(a); assert.equal(b.saved.revision,8); assert.equal(b.saved.state.metals[0].reserve_milli,1000);
  assert.equal(b.inventory.v.items[0].v.count.v,2); assert.equal(b.saved.ledger[0].transaction,'native-roundtrip-7');
  for (const field of ['inventory','saved','foreign','stateMetadata','envelopeOther']) {
    const mixed=clone(b); mixed[field]=field==='foreign'?{x:{t:3,v:6}}:field==='stateMetadata'?{}:field==='envelopeOther'?{}:a[field];
    assert.throws(()=>requireEqual(mixed,b));
  }
  assert.throws(()=>candidate(b)); assert.throws(()=>requireEqual(a,b));
  assert.equal(reconnect(b).saved.revision,9);
  console.log('writer self-test PASS (derived B and mixed-state rejection)');
}
function main(args) {
  if (args.length===1 && args[0]==='--self-test') return selftest();
  if (args.length%2) throw Error('arguments require values');
  const o={}; for(let i=0;i<args.length;i+=2) {if(Object.hasOwn(o,args[i]))throw Error('duplicate argument');o[args[i]]=args[i+1];}
  const required=['--baseline','--world','--owner','--world-id','--mode'];
  if(required.some(k=>!o[k]) || Object.keys(o).some(k=>![...required,'--expect','--baseline-old'].includes(k)) ||
     !/^[0-9a-f]{8}-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}$/.test(o['--owner']) || !/^[0-9a-f]{32}$/.test(o['--world-id']) ||
     !['baseline','write-phase','after-primary-to-old','after-new-to-primary','reopen'].includes(o['--mode'])) throw Error('invalid arguments');
  const mode=o['--mode'], owner=o['--owner'], worldId=o['--world-id'], baseline=readNbt(o['--baseline']),a=extract(baseline,owner,worldId),b=candidate(a);
  let old=null;
  if(o['--baseline-old']) {old=extract(readNbt(o['--baseline-old']),owner,worldId);if(!eq(old,a)&&!eq(reconnect(old),a))throw Error('baseline old not A or predecessor');}
  if(mode==='reopen' && !['A','B'].includes(o['--expect']))throw Error('reopen requires --expect A or B');
  const selected=o['--expect']==='B'?b:a;
  const expected=mode==='baseline'?{'.dat':a,'.dat_old':old,'.dat_new':null}:
    mode==='write-phase'?{'.dat':a,'.dat_old':old,'.dat_new':b}:
    mode==='after-primary-to-old'?{'.dat':null,'.dat_old':a,'.dat_new':b}:
    mode==='after-new-to-primary'?{'.dat':b,'.dat_old':a,'.dat_new':null}:null;
  const snapshots=[];
  for(const suffix of ['.dat','.dat_old','.dat_new']) {
    const file=path.join(o['--world'],'players','data',owner+suffix),present=fs.existsSync(file);
    if(expected && present!==!!expected[suffix])throw Error('unexpected presence: '+suffix);
    if(mode==='reopen' && suffix==='.dat'&&!present)throw Error('reopen missing primary');
    if(!present){snapshots.push({file,present});continue;}
    const parsed=readNbt(file),actual=extract(parsed,owner,worldId);
    if(expected)requireEqual(actual,expected[suffix]);
    else if(suffix==='.dat')requireEqual(actual,reconnect(selected));
    else if(![a,b,reconnect(selected),old].filter(Boolean).some(v=>eq(v,actual)))throw Error('reopen backup differs');
    snapshots.push({file,present,sha256:parsed.sha256,matches:[['A',a],['B',b],['baseline-old',old],['reconnected-'+o['--expect'],reconnect(selected)]].filter(([,v])=>v&&eq(actual,v)).map(([k])=>k)});
  }
  console.log(JSON.stringify({oracle:'PASS',mode,transaction:'native-roundtrip-'+a.saved.revision,baseline:{file:o['--baseline'],sha256:baseline.sha256,revision:a.saved.revision},candidate_revision:b.saved.revision,snapshots,scope:'Complete SavedPlayer, inventory, NeoForgeData namespace and envelope metadata; no arbitrary player motion comparison'},null,2));
}
if(require.main===module) {try{main(process.argv.slice(2));}catch(e){console.error('writer oracle FAIL: '+e.message);process.exitCode=1;}}
module.exports={readNbt,extract,candidate,reconnect,requireEqual};
