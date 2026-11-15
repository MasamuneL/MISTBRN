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
const reference=readNbt(path.join(base,'game-p2-owner-econ/saves/credit/players/data/3efd2c0e-f0de-3e85-b045-dec265b3f79c.dat')).root;
const before=JSON.parse(reference.NeoForgeData['allomancy:state'].payload).saved_player;
const snapshots=[];
for(const kind of ['death-respawn','save-pause2','dimension','death-replay']) {
 const world='game-p2-lifecycle/saves/'+kind,dir=path.join(base,world,'players/data');
 for(const name of fs.readdirSync(dir).filter(n=>/\.dat(?:_old)?$/.test(n)).sort()) {
  const r=readNbt(path.join(dir,name)),ns=r.root.NeoForgeData['allomancy:state'],saved=JSON.parse(ns.payload).saved_player;
  if(ns.format_version!==1||uuid(r.root.UUID)!==before.state.player_id||saved.state.player_id!==before.state.player_id||saved.state.world_id!==before.state.world_id)throw Error('identity/transport '+world);
  if(JSON.stringify(saved.ledger)!==JSON.stringify(before.ledger))throw Error('ledger changed '+world);
  if(saved.state.metals.some((m,i)=>m.reserve_milli!==(kind.startsWith('death-')?0:before.state.metals[i].reserve_milli)||m.unlocked!==before.state.metals[i].unlocked||m.burn!=='Off'))throw Error('metals '+world);
  if(JSON.stringify(saved.rules)!==JSON.stringify(before.rules)||JSON.stringify(saved.accounting)!==JSON.stringify(before.accounting)||saved.state.selected_pair!==before.state.selected_pair||saved.state.last_tick!==null)throw Error('state/accounting '+world);
  if(r.root.NeoForgeData['allomancy:storage_fault']!==undefined)throw Error('fault '+world);
  if(JSON.stringify(r.root.Inventory)!==JSON.stringify(kind.startsWith('death-')?[]:reference.Inventory))throw Error('inventory changed '+world);
  if(r.root.Dimension!==(kind==='dimension'?'minecraft:the_nether':'minecraft:overworld')||r.root.Health!==20)throw Error('dimension/health '+world);
  snapshots.push({world,name,sha256:r.hash,saved,inventory:r.root.Inventory,health:r.root.Health,dimension:r.root.Dimension});
 }
 const run=kind==='death-replay'?'owner-death-replay1':kind==='save-pause2'?'owner-save-pause2':kind==='dimension'?'owner-dimension1':'owner-death-respawn1';
 const receipt=JSON.parse(fs.readFileSync(path.join(base,kind==='death-replay'?'native-credit-runs':'native-lifecycle-runs',run,'result.json')));
 if(!receipt.pass||receipt.exit_code!==0||receipt.termination!=='spontaneous_after_quit')throw Error('receipt '+run);
 if(kind==='save-pause2'&&receipt.save_observed_delta!==2)throw Error('save delta');
}
console.log(JSON.stringify({oracle:'PASS',baseline:'game-p2-owner-econ/saves/credit',snapshots},null,2));