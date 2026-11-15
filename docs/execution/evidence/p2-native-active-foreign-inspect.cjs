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
const key=Buffer.from('foreign:keep'), hdr=Buffer.alloc(3);hdr[0]=3;hdr.writeUInt16BE(key.length,1);const seven=Buffer.alloc(4);seven.writeInt32BE(7);const sig=Buffer.concat([hdr,key,seven]);
for(const kind of ['credit','quarantine']) {
 const world='game-p2-active-foreign/saves/'+kind,dir=path.join(base,world,'players/data');
 for(const name of fs.readdirSync(dir).filter(n=>/\.dat(?:_old)?$/.test(n)).sort()) {
  const file=path.join(dir,name),original=fs.readFileSync(file),r=readNbt(file),mods=r.root.NeoForgeData;
  const saved=JSON.parse(mods['allomancy:state'].payload.toString('utf8')).saved_player;
  const raw=zlib.gunzipSync(original),at=raw.indexOf(sig);
  if(at<0||raw.indexOf(sig,at+1)!==-1||mods['foreign:keep']!==7)throw Error('foreign Int7 not preserved');
  if(uuid(r.root.UUID)!=='3efd2c0ef0de3e85b045dec265b3f79c'||saved.state.player_id!==uuid(r.root.UUID))throw Error('identity');
  const q=kind==='quarantine';
  if(saved.state.metals[0].reserve_milli!==(q?0:1000)||saved.ledger.length!==(q?0:1))throw Error('economy');
  if(r.root.Inventory.length!==1||r.root.Inventory[0].id!=='minecraft:iron_ingot'||r.root.Inventory[0].count!==2)throw Error('inventory');
  const fault=mods['allomancy:storage_fault'];
  if(q?fault?.format_version!==1||fault?.reason!=='rollback_unverified':fault!==undefined)throw Error('quarantine');
  console.log(JSON.stringify({world,name,sha256:r.hash,foreign_keep:7,nbt_type:'Int',reserve:saved.state.metals[0].reserve_milli,ledger:saved.ledger,revision:saved.revision,marked:!!fault}));
 }
}
console.error('ORACLE PASS foreign active player credit+quarantine four snapshots');