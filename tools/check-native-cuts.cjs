'use strict';
const fs=require('fs'),path=require('path'),crypto=require('crypto'),assert=require('assert');
const runtime=process.argv[2];
if(!runtime)throw Error('usage: node check-native-cuts.cjs RUNTIME');
const repo=path.resolve(__dirname,'..');
const read=p=>JSON.parse(fs.readFileSync(p,'utf8').replace(/^\uFEFF/,''));
const hash=p=>crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex').toUpperCase();
const cases=[];
for(const frontier of ['before-inventory','after-inventory','after-inline','after-core-commit']){
 const run='cuts-'+frontier+'-1',dir=path.join(runtime,'native-cut-runs',run),file=path.join(dir,'result.json'),r=read(file);
 const reopenFile=path.join(runtime,'native-open-runs',run+'-reopen','result.json'),o=read(reopenFile);
 const oracleFile=path.join(repo,'docs/execution/evidence',run+'-oracle.json'),reopenOracleFile=path.join(repo,'docs/execution/evidence',run+'-reopen-oracle.json');
 const a=read(oracleFile),b=read(reopenOracleFile);
 assert(r.pass&&r.status==='frontier_captured'&&r.termination==='killed_at_exact_adapter_frontier'&&r.frontier===frontier&&r.exit_code!==null);
 assert(r.engine_sha256==='8C593D4DFBFDDF5054C8FC94DE860A27B945542584488BF42E5412E5766CFA70');
 const trace=`ALLOMANCY_NATIVE_CUT event=paused pid=${r.pid} frontier=${frontier} transaction=${r.transaction} run_token=${r.run_token} durable=false`;
 const lines=fs.readFileSync(r.stderr,'utf8').split(/\r?\n/);
 assert(r.transaction==='native-cut-'+r.run_token&&r.trace===trace&&lines.filter(x=>x===trace).length===1&&lines[r.trace_line-1]===trace);
 assert(!lines.slice(r.trace_line).some(x=>/ALLOMANCY_(SAVE|NATIVE_TEST|NATIVE_CUT)/.test(x)),'adapter continued after pause');
 assert(!lines.some(x=>/result=FAIL|ALLOMANCY_SAVE event=fault/.test(x)));
 assert(r.baseline_save.exit===0&&JSON.parse(r.baseline_save.reply).ok===true);
 for(const snap of [...r.baseline,...r.after])assert(hash(snap.copy)===snap.sha256);
 assert(o.pass&&o.exit_code===0&&o.termination==='spontaneous'&&o.world!==r.world);
 assert(a.oracle==='PASS'&&b.oracle==='PASS'&&a.expect==='A'&&b.expect==='A'&&a.transaction===r.transaction&&b.transaction===r.transaction);
 for(const kind of ['clone','reopen-clone']){const c=read(path.join(repo,'docs/execution/evidence',run+'-'+kind+'.json'));assert(c.closed&&c.equal);assert.deepStrictEqual(c.source_files,c.copied_files);}
 cases.push({run,frontier,pid:r.pid,transaction:r.transaction,trace,cut_exit:r.exit_code,termination:r.termination,cut_receipt_sha256:hash(file),reopen_receipt_sha256:hash(reopenFile),oracle_sha256:hash(oracleFile),reopen_oracle_sha256:hash(reopenOracleFile),cut:'A complete',reopen:'reconnected A complete',pass:true});
}
console.log(JSON.stringify({build:'p2-cuts1',engine_sha256:'8C593D4DFBFDDF5054C8FC94DE860A27B945542584488BF42E5412E5766CFA70',new_receipts:8,prior_receipts:39,total_observed:47,pass:true,n6_scope:'four adapter frontiers in integrated owner no-save parked profile',p2_complete:false,n7:false,fsync:false,cases},null,2));
