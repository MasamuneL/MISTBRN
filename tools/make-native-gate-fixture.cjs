'use strict';
// Offline only: caller must close the game and use a full, disposable world clone.
const fs=require('node:fs'), path=require('node:path'), zlib=require('node:zlib'), crypto=require('node:crypto');
const owner='3efd2c0ef0de3e85b045dec265b3f79c', worldId='026e465a87434711a6951c07588dc0ee';
const stem='3efd2c0e-f0de-3e85-b045-dec265b3f79c';
const kinds=['both-owner-invalid','future-owner-primary-valid-old','player-seed','dead-player','capacity-player','ledger-full-player'];
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const i32=n=>{const b=Buffer.alloc(4);b.writeInt32BE(n);return b};
function parse(original) {
 const raw=zlib.gunzipSync(original,{maxOutputLength:32*1024*1024}); let at=0; const tags={};
 const take=n=>{if(n<0||at+n>raw.length)throw Error('NBT length');let b=raw.subarray(at,at+n);at+=n;return b};
 const u8=()=>take(1)[0], num=()=>take(4).readInt32BE(), str=()=>take(take(2).readUInt16BE()).toString('utf8');
 const count=()=>{let n=num();if(n<0||n>10000000)throw Error('NBT count');return n};
 function value(t,route=[],depth=0){if(depth>128)throw Error('NBT depth');switch(t){
 case 1:return take(1).readInt8();case 2:return take(2).readInt16BE();case 3:return num();case 4:return take(8).readBigInt64BE();case 5:return take(4).readFloatBE();case 6:return take(8).readDoubleBE();case 7:return take(count());case 8:return str();
 case 9:{const c=u8(),n=count();return Array.from({length:n},()=>value(c,[...route,'[]'],depth+1))}
 case 10:{let out=Object.create(null);for(;;){let start=at,c=u8();if(!c)return out;let k=str();if(Object.hasOwn(out,k))throw Error('duplicate NBT');let vstart=at;out[k]=value(c,[...route,k],depth+1);tags[[...route,k].join('/')]= {start,vstart,end:at,type:c};}}
 case 11:return Array.from({length:count()},num);case 12:return Array.from({length:count()},()=>take(8).readBigInt64BE());default:throw Error('NBT type '+t)}}
 if(u8()!==10)throw Error('root is not compound');str();let root=value(10);if(at!==raw.length)throw Error('NBT trailing bytes');return {raw,root,tags};
}
const uuid=a=>Array.isArray(a)&&a.length===4?a.map(n=>(n>>>0).toString(16).padStart(8,'0')).join(''):null;
function replace(raw,changes){for(const c of changes.sort((a,b)=>b.start-a.start))raw=Buffer.concat([raw.subarray(0,c.start),c.bytes,raw.subarray(c.end)]);return zlib.gzipSync(raw)}
function regular(p){let s=fs.lstatSync(p);if(!s.isFile()||s.isSymbolicLink()||fs.realpathSync(p)!==p)throw Error('not a real regular file: '+p)}
function directory(p){let s=fs.lstatSync(p);if(!s.isDirectory()||s.isSymbolicLink()||fs.realpathSync(p)!==p)throw Error('not a real directory: '+p)}
function validateSnapshot(b){let n=parse(b), state=n.root.NeoForgeData?.['allomancy:state'];if(uuid(n.root.UUID)!==owner||state?.format_version!==1||!Buffer.isBuffer(state.payload))throw Error('owner state missing');let doc=JSON.parse(state.payload.toString('utf8'));if(doc.schema_version!==1||doc.saved_player?.state?.player_id!==owner||doc.saved_player.state.world_id!==worldId)throw Error('owner payload identity mismatch');return {n,doc}}
function make(kind,b){let {n,doc}=validateSnapshot(b), edits=[];
 if(kind==='both-owner-invalid') return b.subarray(0,10);
 if(kind==='future-owner-primary-valid-old')doc.schema_version=999;
 else {let s=doc.saved_player;s.state.selected_pair='CopperBronze';s.state.last_tick=19;s.state.metals=Array.from({length:8},(_,i)=>({unlocked:true,reserve_milli:100+i,burn:i%2?'Flared':'Burning'}));s.accounting.remainder=Array.from({length:8},(_,i)=>123+i);s.rules={consumption_milli_per_tick:Array(8).fill(1),base_capacity_milli:Array(8).fill(10000),reduction_milli:200,capacity_bonus_milli:0};s.revision=1;s.ledger=[{transaction:'fixture-drink-1',payload:{metal:'Iron',amount_milli:37},committed_revision:1}];
 if(kind==='capacity-player')s.state.metals[0].reserve_milli=10000;
 if(kind==='ledger-full-player'){s.revision=4096;s.ledger=Array.from({length:4096},(_,i)=>({transaction:'fixture-'+String(i).padStart(4,'0'),payload:{metal:'Iron',amount_milli:0},committed_revision:i+1}));}
 if(kind==='dead-player'){let t=n.tags.Health;if(!t||t.type!==5)throw Error('Health float missing');edits.push({start:t.vstart,end:t.end,bytes:Buffer.alloc(4)})}}
 let t=n.tags['NeoForgeData/allomancy:state/payload'];if(!t||t.type!==7)throw Error('payload byte array missing');let payload=Buffer.from(JSON.stringify(doc));edits.push({start:t.vstart,end:t.end,bytes:Buffer.concat([i32(payload.length),payload])});return replace(n.raw,edits)
}
function main(){let a=process.argv.slice(2);if(a.length!==4||a[0]!=='--world'||a[2]!=='--kind'||!kinds.includes(a[3]))throw Error('Usage: node make-native-gate-fixture.cjs --world <world> --kind <'+kinds.join('|')+'>');
 let world=path.resolve(a[1]), saves=path.dirname(world),game=path.dirname(saves),runtime=path.dirname(game);if(path.basename(saves)!=='saves'||path.basename(game)!=='game-p2-gates'||path.basename(runtime)!=='runtime'||!path.basename(world)||path.basename(world)==='.'||path.basename(world)==='..')throw Error('expected runtime/game-p2-gates/saves/NAME');
 // Reject redirections at every existing component, including ancestors.
 let parts=[];for(let p=world;;p=path.dirname(p)){parts.push(p);if(path.dirname(p)===p)break}for(let p of parts.reverse())directory(p);
 let data=path.join(world,'players','data');directory(path.join(world,'players'));directory(data);
 let level=path.join(world,'level.dat');regular(level);if(uuid(parse(fs.readFileSync(level)).root.Data?.singleplayer_uuid)!==owner)throw Error('unexpected level owner');
 let marker=path.join(world,'allomancy','world-identity.json');directory(path.dirname(marker));regular(marker);let id=JSON.parse(fs.readFileSync(marker,'utf8'));if(id.schema_version!==1||id.world_id!==worldId)throw Error('unexpected world identity');
 let names=[stem+'.dat',stem+'.dat_old'], items=names.map(name=>{let file=path.join(data,name);regular(file);let before=fs.readFileSync(file);validateSnapshot(before);return {name,file,before}});
 let kind=a[3];for(let [i,item] of items.entries())item.after=(kind==='future-owner-primary-valid-old'&&i===1)?item.before:make(kind,item.before);
 let backup=path.join(world,'.fixture-original');fs.mkdirSync(backup);for(let x of items)fs.writeFileSync(path.join(backup,x.name),x.before,{flag:'wx'});
 for(let x of items){if(!x.after.equals(x.before))fs.writeFileSync(x.file,x.after);if(!fs.readFileSync(x.file).equals(x.after))throw Error('write verification failed');if(kind!=='both-owner-invalid'&&kind!=='future-owner-primary-valid-old')validateSnapshot(x.after)}
 console.log(JSON.stringify({kind,owner,world_id:worldId,files:items.map(x=>({path:x.file,backup:path.join(backup,x.name),before_sha256:sha(x.before),after_sha256:sha(x.after)}))}));
}
function selfTest(){
 const assert=require('node:assert/strict');
 const nm=s=>{let b=Buffer.from(s),n=Buffer.alloc(2);n.writeUInt16BE(b.length);return Buffer.concat([n,b])};
 const tag=(t,k,b)=>Buffer.concat([Buffer.from([t]),nm(k),b]);
 const doc={schema_version:1,saved_player:{state:{player_id:owner,world_id:worldId,metals:[]},accounting:{remainder:[]},rules:{},revision:0,ledger:[]}};
 const payload=Buffer.from(JSON.stringify(doc));const u=Buffer.concat([i32(4),...owner.match(/.{8}/g).map(x=>i32(parseInt(x,16)|0))]);
 const raw=Buffer.concat([Buffer.from([10,0,0]),tag(11,'UUID',u),tag(5,'Health',Buffer.from([0x41,0xa0,0,0])),tag(10,'NeoForgeData',Buffer.concat([tag(3,'foreign:keep',i32(7)),tag(10,'allomancy:state',Buffer.concat([tag(3,'format_version',i32(1)),tag(7,'payload',Buffer.concat([i32(payload.length),payload])),Buffer.from([0])])),Buffer.from([0])])),Buffer.from([0])]);
 const original=zlib.gzipSync(raw);assert.equal(validateSnapshot(original).n.root.NeoForgeData['foreign:keep'],7);
 for(const kind of kinds){let out=make(kind,original);if(kind==='both-owner-invalid'){assert.throws(()=>parse(out));continue}let n=parse(out), state=n.root.NeoForgeData['allomancy:state'],d=JSON.parse(state.payload);assert.equal(n.root.NeoForgeData['foreign:keep'],7);assert.deepEqual(n.root.UUID,parse(original).root.UUID);if(kind==='future-owner-primary-valid-old')assert.equal(d.schema_version,999);else{assert.equal(d.saved_player.state.metals.length,8);assert.equal(d.saved_player.accounting.remainder[0],123);if(kind==='dead-player')assert.equal(n.root.Health,0);if(kind==='capacity-player')assert.equal(d.saved_player.state.metals[0].reserve_milli,10000);if(kind==='ledger-full-player')assert.equal(d.saved_player.ledger.length,4096)}}
 assert.throws(()=>parse(Buffer.from([1,2,3])));console.log('self-test PASS');
}
try{if(process.argv.length===3&&process.argv[2]==='--self-test')selfTest();else main()}catch(e){console.error('fixture error: '+e.message);process.exitCode=1}


