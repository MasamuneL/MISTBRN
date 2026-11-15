const fs=require('fs'),path=require('path'),zlib=require('zlib'),crypto=require('crypto');
const base='C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime';
function readNbt(file){
 const original=fs.readFileSync(file);
 const bytes=original[0]===31&&original[1]===139?zlib.gunzipSync(original):original;
 let at=0;
 const take=n=>{if(n<0||at+n>bytes.length)throw Error('NBT length');const b=bytes.subarray(at,at+n);at+=n;return b;};
 const u8=()=>take(1)[0],i32=()=>take(4).readInt32BE(),str=()=>take(take(2).readUInt16BE()).toString('utf8');
 const count=()=>{const n=i32();if(n<0||n>10000000)throw Error('NBT count');return n;};
 function value(type,depth=0){
 if(depth>512)throw Error('NBT depth');
 switch(type){
 case 1:return take(1).readInt8();case 2:return take(2).readInt16BE();case 3:return i32();case 4:return take(8).readBigInt64BE();case 5:return take(4).readFloatBE();case 6:return take(8).readDoubleBE();case 7:return take(count());case 8:return str();
 case 9:{const child=u8(),n=count();return Array.from({length:n},()=>value(child,depth+1));}
 case 10:{const out=Object.create(null);while(true){const child=u8();if(!child)return out;const key=str();if(Object.hasOwn(out,key))throw Error('duplicate NBT');out[key]=value(child,depth+1);}}
 case 11:return Array.from({length:count()},i32);case 12:return Array.from({length:count()},()=>take(8).readBigInt64BE());default:throw Error('NBT type '+type);
 }}
 const type=u8();str();const result=value(type);if(at!==bytes.length)throw Error('NBT trailing bytes');
 return {root:result,hash:crypto.createHash('sha256').update(original).digest('hex').toUpperCase()};
}
const uuid=ints=>Array.isArray(ints)&&ints.length===4?ints.map(n=>(n>>>0).toString(16).padStart(8,'0')).join(''):null;
function player(t){
 if(!t)return null;
 const namespace=t.NeoForgeData?.['allomancy:state'];
 const payload=Buffer.isBuffer(namespace?.payload)?JSON.parse(namespace.payload.toString('utf8')):null;
 return {nbtUuid:uuid(t.UUID),payloadPlayer:payload?.saved_player?.state?.player_id??null,payloadWorld:payload?.saved_player?.state?.world_id??null,storageFaultPresent:Object.hasOwn(t.NeoForgeData??{},'allomancy:storage_fault')};
}
const snapshots=[];
for(const kind of ['create-new','deny-rename']) {
 const world='game-p2-io/saves/'+kind,dir=path.join(base,world,'players/data');
 const receipt=JSON.parse(fs.readFileSync(path.join(base,'native-lifecycle-runs','owner-io-'+kind+'1/result.json')));
 if(!receipt.pass||receipt.termination!=='killed_after_expected_io_error'||!/os error 32\)/.test(receipt.io_error))throw Error('receipt IO');
 if(receipt.io_before.length!==2||receipt.io_after.length!==2)throw Error('snapshots count');
 for(const suffix of ['.dat','.dat_old','.dat_new']) {
  const file=path.join(dir,'3efd2c0e-f0de-3e85-b045-dec265b3f79c'+suffix),raw=fs.readFileSync(file);
  if(suffix==='.dat_new'&&kind==='create-new') {
   if(raw.length!==0)throw Error('create-denied temporal modified');
   snapshots.push({world,suffix,bytes:0,interpretation:'empty harness-created locked file; not a game snapshot'});continue;
  }
  const r=readNbt(file),namespace=r.root.NeoForgeData?.['allomancy:state'],payload=JSON.parse(namespace.payload),saved=payload.saved_player;
  if(namespace.format_version!==1||payload.schema_version!==1||uuid(r.root.UUID)!=='3efd2c0ef0de3e85b045dec265b3f79c'||saved.state.player_id!==uuid(r.root.UUID)||saved.state.world_id!=='026e465a87434711a6951c07588dc0ee')throw Error('identity');
  if(saved.state.metals[0].reserve_milli!==1000||saved.ledger.length!==1||saved.ledger[0].transaction!=='native-roundtrip-3'||saved.ledger[0].committed_revision!==4||saved.ledger[0].payload?.metal!=='Iron'||saved.ledger[0].payload?.amount_milli!==1000||saved.revision!==5||r.root.NeoForgeData['allomancy:storage_fault']!==undefined)throw Error('economy');
  const count=suffix==='.dat_new'?3:2;
  if(r.root.Inventory.length!==1||r.root.Inventory[0].id!=='minecraft:iron_ingot'||r.root.Inventory[0].count!==count)throw Error('inventory');
  if(suffix!=='.dat_new') {
   const expected=receipt.io_before.find(s=>s.path.endsWith(suffix));
   if(!expected||expected.sha256!==r.hash||receipt.io_after.find(s=>s.path===expected.path)?.sha256!==r.hash)throw Error('before/after hash');
  }
  snapshots.push({world,suffix,sha256:r.hash,reserve:1000,revision:5,ledger:saved.ledger,inventory:r.root.Inventory,interpretation:suffix==='.dat_new'?'complete attempted B, never adopted':'complete retained A'});
 }
}
console.log(JSON.stringify({oracle:'PASS',snapshots},null,2));