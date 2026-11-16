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
function main(args){
 if(args.length===1&&args[0]==='--self-test'){
  const owner='3efd2c0e-f0de-3e85-b045-dec265b3f79c',wid='026e465a87434711a6951c07588dc0ee';
  const saved={state:{player_id:owner.replaceAll('-',''),world_id:wid,metals:Array.from({length:8},()=>({reserve_milli:1,burn:'Off'}))},revision:1,ledger:[]};
  const payload=Buffer.from(JSON.stringify({schema_version:1,saved_player:saved})).toString('hex');
  const parsed={root:{v:{UUID:{t:11,v:[0x3efd2c0e,0xf0de3e85|0,0xb045dec2|0,0x65b3f79c]},Inventory:{t:9,v:{child:0,items:[]}},NeoForgeData:{t:10,v:{'allomancy:state':{t:10,v:{format_version:{t:3,v:1},payload:{t:7,v:payload}}}}}}}};
  assert.equal(extract(parsed,owner,wid).inventory.v.child,0);
  parsed.root.v.Inventory.v={child:1,items:[]};assert.throws(()=>extract(parsed,owner,wid));
  parsed.root.v.Inventory.v={child:0,items:[{}]};assert.throws(()=>extract(parsed,owner,wid));
  const a={inventory:{t:9,v:{child:10,items:[{t:10,v:{item:{t:8,v:'item'}}}]}},saved:{state:{metals:Array.from({length:8},()=>({unlocked:true,reserve_milli:5,burn:'Burning'})),last_tick:19},accounting:{remainder:Array(8).fill(3)},rules:{x:1},revision:1,ledger:[{x:1}]},foreign:{x:{t:3,v:7}},stateMetadata:{format_version:{t:3,v:1}},envelopeOther:{schema_version:1}};
  for(const mode of ['unchanged','identity','reconnect','death-login','death-respawn']){const b=expected(a,mode);verify(b,expected(a,mode));let bad=clone(b);bad.saved.revision++;assert.throws(()=>verify(bad,expected(a,mode)));bad=clone(b);bad.foreign.x.v++;assert.throws(()=>verify(bad,expected(a,mode)))}
  assert.equal(expected(a,'death-login').saved.revision,3);assert.equal(expected(a,'death-respawn').inventory.v.items.length,0);assert.equal(expected(a,'death-respawn').inventory.v.child,0);assert.equal(expected(a,'reconnect').saved.accounting.remainder[0],3);
  console.log('self-test PASS');return;
 }
 if(args.length%2)throw Error('arguments require values');let o={};for(let i=0;i<args.length;i+=2){if(!args[i].startsWith('--')||Object.hasOwn(o,args[i]))throw Error('invalid argument');o[args[i]]=args[i+1]}
 const required=['--baseline','--world','--owner','--world-id','--mode'];if(required.some(k=>!o[k])||Object.keys(o).some(k=>![...required,'--old-expect'].includes(k))||!/^[0-9a-f]{8}-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}$/.test(o['--owner'])||!/^[0-9a-f]{32}$/.test(o['--world-id'])||!['identity','unchanged','reconnect','death-login','death-respawn'].includes(o['--mode'])||(o['--old-expect']&&!['baseline','reconnect','same'].includes(o['--old-expect'])))throw Error('invalid arguments');
 let baseline=readNbt(o['--baseline']), a=extract(baseline,o['--owner'],o['--world-id']), target=expected(a,o['--mode']),dir=path.join(o['--world'],'players','data'),results=[];
 for(const suffix of ['.dat','.dat_old','.dat_new']){let file=path.join(dir,o['--owner']+suffix);if(!fs.existsSync(file)){if(suffix==='.dat')throw Error('primary missing');results.push({file,present:false});continue}if(suffix==='.dat_new')throw Error('unexpected .dat_new');if(suffix==='.dat_old'&&!o['--old-expect'])throw Error('old-expect required');let parsed=readNbt(file),actual=extract(parsed,o['--owner'],o['--world-id']);let exp=suffix==='.dat'?target:o['--old-expect']==='baseline'?a:o['--old-expect']==='reconnect'?expected(a,'reconnect'):target;verify(actual,exp);results.push({file,present:true,sha256:parsed.sha256,expected:suffix==='.dat'?o['--mode']:o['--old-expect']})}
 console.log(JSON.stringify({oracle:'PASS',mode:o['--mode'],baseline:{file:o['--baseline'],sha256:baseline.sha256},snapshots:results},null,2));
}
try{main(process.argv.slice(2))}catch(e){console.error('oracle FAIL: '+e.message);process.exitCode=1}



