# Bounded N5 observer: no launch, suspend, detach, continue or kill performed here.
set pagination off
set width 0
set language c
set non-stop off
python
import _gdb as gdb
import os, json, hashlib, re
n5 = None
n5_breaks = []
def n5_reg(name):
    return int(gdb.parse_and_eval('$' + name))
def n5_mem(address, size):
    if size < 0 or size > 16777216:
        raise RuntimeError('Invalid buffer length')
    return bytes(gdb.selected_inferior().read_memory(address, size))
def n5_u64(address):
    return int.from_bytes(n5_mem(address, 8), 'little')
def n5_norm(path):
    return os.path.normcase(os.path.normpath(path))
def n5_address(symbol):
    found = re.search(r' is at (0x[0-9a-fA-F]+)', gdb.execute('info address '+symbol, to_string=True))
    if not found:
        raise RuntimeError('Unresolved symbol '+symbol)
    return int(found.group(1),16)
def n5_publish(suffix, payload):
    path = n5['receipt']+suffix
    if os.path.exists(path) or os.path.exists(path+'.tmp'):
        raise RuntimeError('Receipt already exists')
    with open(path+'.tmp','x',encoding='utf-8') as stream:
        json.dump(payload,stream,indent=2)
        stream.flush()
    os.replace(path+'.tmp',path)
class N5Checked(gdb.Breakpoint):
    def stop(self):
        try:
            return self.hit()
        except Exception as error:
            print('N5_DEBUG_ERROR '+repr(error),flush=True)
            return True
class N5Entry(N5Checked):
    def hit(self):
        size=n5_reg('rdx')
        if not 0<size<32768:
            return False
        path=n5_mem(n5_reg('rcx'),size).decode('utf-8')
        if n5_norm(path)!=n5_norm(n5['owner']):
            return False
        if n5_reg('r9')&255:
            raise RuntimeError('Unexpected durable writer branch')
        if n5.get('thread') is not None:
            raise RuntimeError('Second target entry before capture')
        n5['thread']=gdb.selected_thread().num
        n5['entry_ptid']=list(gdb.selected_thread().ptid)
        return False
class N5NtWrite(N5Checked):
    def hit(self):
        if gdb.selected_thread().num!=n5.get('thread'):
            return False
        sp=n5_reg('rsp')
        buffer=n5_u64(sp+48)
        size=n5_u64(sp+56)&0xffffffff
        if not 0<size<=16777216:
            return False
        payload=n5_mem(buffer,size)
        if not payload.startswith(b'\x1f\x8b'):
            return False
        inferior=gdb.selected_inferior()
        threads=[{'num':t.num,'ptid':list(t.ptid),'stopped':t.is_stopped()} for t in inferior.threads()]
        if gdb.parameter('non-stop') or not all(t['stopped'] for t in threads):
            raise RuntimeError('Inferior is not all-stop')
        receipt={'schema':1,'boundary':'NtWriteFile-call-entry-before-completion',
                 'pid':inferior.pid,'ptid':list(gdb.selected_thread().ptid),
                 'thread':gdb.selected_thread().num,'thread_name':gdb.selected_thread().name,
                 'pc':hex(n5_reg('pc')),'handle':n5_reg('rcx'),
                 'buffer':buffer,'length':size,'gzip_sha256':hashlib.sha256(payload).hexdigest(),
                 'owner':n5['owner'],'run_token':n5['token'],'engine_sha256':n5['hash'],
                 'all_stop':True,'threads':threads,
                 'limitation':'API call entry, not successful or interrupted kernel IO; controller must verify duplicated handle target and thread owner/name before suspension'}
        n5_publish('',receipt)
        for breakpoint in n5_breaks:
            breakpoint.enabled=False
        print('N5_WRITER_HELD '+json.dumps(receipt),flush=True)
        return True
class N5Arm(gdb.Command):
    def __init__(self):
        super().__init__('fifo-arm',gdb.COMMAND_USER)
    def invoke(self,args,from_tty):
        global n5
        values=gdb.string_to_argv(args)
        if len(values)!=6:
            raise RuntimeError('fifo-arm PID OWNER_DAT ENGINE_SHA256 SYMBOL TOKEN NEW_RECEIPT')
        pid,owner,pin,symbol,token,receipt=values
        inferior=gdb.selected_inferior()
        if n5 is not None or inferior.pid!=int(pid) or inferior.pid<=0:
            raise RuntimeError('Already armed or PID mismatch')
        if not re.fullmatch(r'[a-fA-F0-9]{64}',pin) or not re.fullmatch(r'[A-Za-z0-9_-]{1,64}',token):
            raise RuntimeError('Invalid pin/token')
        if not re.fullmatch(r'[A-Za-z0-9_.$]+',symbol) or 'write_dat_as' not in symbol:
            raise RuntimeError('Invalid symbol')
        if not os.path.isabs(owner) or not os.path.isabs(receipt) or not owner.endswith('.dat') or not os.path.isfile(owner):
            raise RuntimeError('Absolute existing guest DAT and new receipt required')
        engine=inferior.progspace.filename
        with open(engine,'rb') as stream:
            actual=hashlib.sha256(stream.read()).hexdigest()
        if actual!=pin.lower() or gdb.parameter('non-stop'):
            raise RuntimeError('Wrong engine or non-stop mode')
        base=n5_address(symbol)
        ntwrite=n5_address('NtWriteFile')
        n5={'owner':owner,'receipt':receipt,'token':token,'hash':actual}
        n5_breaks.append(N5Entry('*'+hex(base),internal=True))
        n5_breaks.append(N5NtWrite('*'+hex(ntwrite),internal=True))
        n5_publish('.armed.json',{'pid':inferior.pid,'owner':owner,'engine_sha256':actual,'run_token':token,'symbol':symbol,'base':hex(base),'ntwrite':hex(ntwrite)})
        print('N5_ARMED',flush=True)
N5Arm()
end