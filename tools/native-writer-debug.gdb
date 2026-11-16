# External all-stop debugger; cuts1 only. Never starts, detaches, or kills a process.
# Root owns launch, baseline, async save, timeout, exact-PID kill and offline oracle.
set pagination off
set width 0
set language c
# Controller sets non-stop off before attach; never change it on a live inferior.
python
import _gdb as gdb
import os, json, hashlib, re

PIN = "8c593d4dfbfddf5054c8fc94de860a27b945542584488bf42e5412e5766cfa70"
SYMBOL = "_RNvNtCskc7bsog4VuE_18horizon_client_lib4save12write_dat_as.llvm.7519682002513022163"
state = None
breaks = []

def reg(name):
    return int(gdb.parse_and_eval("$" + name))

def memory(address, length):
    if length < 0 or length > 16777216:
        raise RuntimeError("Invalid memory length")
    return bytes(gdb.selected_inferior().read_memory(address, length))

def u64(address):
    return int.from_bytes(memory(address, 8), "little")

def norm(path):
    return os.path.normcase(os.path.normpath(path))

def symbol_address(name):
    output = gdb.execute("info address " + name, to_string=True)
    match = re.search(r" is at (0x[0-9a-fA-F]+)", output)
    if not match:
        raise RuntimeError("Cannot map symbol: " + name)
    return int(match.group(1), 16)

def publish(suffix, data):
    path = state["receipt"] + suffix
    if os.path.exists(path):
        raise RuntimeError("Refusing to overwrite receipt")
    temporary = path + ".tmp"
    with open(temporary, "x", encoding="utf-8") as stream:
        json.dump(data, stream, indent=2)
        stream.flush()
    os.replace(temporary, path)

def stop_receipt(boundary, **extra):
    # A receipt is evidence of a stop only; root must observe GDB stopped and kill
    # its owned process before inspecting/copying files. This is never fsync proof.
    payload = dict(schema=1, boundary=boundary, pid=gdb.selected_inferior().pid,
                   thread=state.get("thread"), ptid=list(gdb.selected_thread().ptid),
                   pc=hex(reg("pc")), owner_path=state["owner"],
                   engine_sha256=PIN, write_dat_as=hex(state["base"]),
                   all_stop=not gdb.parameter("non-stop"), **extra)
    payload["threads"] = [{"num": t.num, "ptid": list(t.ptid),
                           "stopped": t.is_stopped()} for t in gdb.selected_inferior().threads()]
    if not all(t["stopped"] for t in payload["threads"]):
        raise RuntimeError("Not all debuggee threads stopped")
    publish("", payload)
    print("N7_BOUNDARY_HELD " + json.dumps(payload))
    return True

class CheckedBreakpoint(gdb.Breakpoint):
    def stop(self):
        try:
            return self.hit()
        except Exception as error:
            # Fail closed: no automatic continue after a decoder/mapping error.
            print("N7_DEBUG_ERROR " + repr(error))
            return True

class Entry(CheckedBreakpoint):
    def hit(self):
        length = reg("rdx")
        if length <= 0 or length > 32768:
            return False
        path = memory(reg("rcx"), length).decode("utf-8")
        if norm(path) != norm(state["owner"]):
            return False
        if reg("r9") & 255:
            raise RuntimeError("Unexpected durable branch")
        if state.get("thread") is not None:
            raise RuntimeError("Second matching writer before requested boundary")
        state["thread"] = gdb.selected_thread().num
        state["ptid"] = list(gdb.selected_thread().ptid)
        print("N7_OWNER_WRITE " + json.dumps({"path":path,"thread":state["thread"]}))
        return False

class WriteCall(CheckedBreakpoint):
    def hit(self):
        if gdb.selected_thread().num != state.get("thread"):
            return False
        path = memory(reg("rcx"), reg("rdx")).decode("utf-8")
        if norm(path) != norm(state["owner"] + "_new"):
            raise RuntimeError("fs::write path mismatch")
        data = memory(reg("r8"), reg("r9"))
        if not data.startswith(b"\x1f\x8b") or len(data) == 0:
            raise RuntimeError("Expected gzip snapshot")
        state["write"] = {"path":path, "buffer":reg("r8"), "length":len(data),
                          "sha256":hashlib.sha256(data).hexdigest()}
        return False

class WriteReturn(CheckedBreakpoint):
    def __init__(self, address, iosb, handle):
        super().__init__("*" + hex(address), internal=True)
        self.thread = state["thread"]
        self.iosb = iosb
        self.handle = handle

    def hit(self):
        status = reg("rax") & 0xffffffff
        ios = memory(self.iosb, 16)
        ios_status = int.from_bytes(ios[:4], "little")
        written = int.from_bytes(ios[8:], "little")
        if status != 0 or ios_status != 0 or written != state["write"]["length"]:
            raise RuntimeError("NtWriteFile not complete success: " + repr((status, ios_status, written)))
        self.enabled = False
        return stop_receipt("ntwrite-success-before-fs-write-close",
                            ntstatus=status, io_status=ios_status, bytes_written=written,
                            handle=hex(self.handle), write=state["write"],
                            limitation="Complete syscall write; no partial-byte or interrupted-kernel-write claim")

class NtWrite(CheckedBreakpoint):
    def hit(self):
        if gdb.selected_thread().num != state.get("thread") or "write" not in state:
            return False
        sp = reg("rsp")
        buffer, length = u64(sp + 48), u64(sp + 56) & 0xffffffff
        candidate = state["write"]
        if buffer != candidate["buffer"] or length != candidate["length"]:
            return False
        if hashlib.sha256(memory(buffer, length)).hexdigest() != candidate["sha256"]:
            raise RuntimeError("NtWriteFile buffer drift")
        breaks.append(WriteReturn(u64(sp), u64(sp + 40), reg("rcx")))
        return False

class RenameReturn(CheckedBreakpoint):
    def hit(self):
        if gdb.selected_thread().num != state.get("thread"):
            return False
        if "write" not in state or reg("rax") != 0:
            raise RuntimeError("Rename without mapped write or successful std::io result")
        return stop_receipt(state["mode"], std_io_result=0, write=state["write"],
                            limitation="Rename returned before write_dat_as returned; no save callback or fsync")

class Arm(gdb.Command):
    def __init__(self):
        super().__init__("writer-arm", gdb.COMMAND_USER)

    def invoke(self, args, from_tty):
        global state
        values = gdb.string_to_argv(args)
        if len(values) != 4:
            raise RuntimeError('writer-arm OWNED_PID "ABS_OWNER_DAT" MODE "ABS_NEW_RECEIPT"')
        pid, owner, mode, receipt = values
        inferior = gdb.selected_inferior()
        if state is not None or inferior.pid != int(pid) or inferior.pid <= 0:
            raise RuntimeError("Already armed or PID mismatch")
        if mode not in ("write-phase", "after-primary-to-old", "after-new-to-primary"):
            raise RuntimeError("Unknown boundary")
        if not os.path.isabs(owner) or not os.path.isabs(receipt) or not owner.endswith(".dat"):
            raise RuntimeError("Require absolute owner.dat and receipt paths")
        if not os.path.isfile(owner) or os.path.exists(receipt) or os.path.exists(receipt+".armed.json"):
            raise RuntimeError("Missing owner baseline or receipt already exists")
        engine = inferior.progspace.filename
        with open(engine, "rb") as stream:
            actual = hashlib.file_digest(stream, "sha256").hexdigest() if hasattr(hashlib, "file_digest") else hashlib.sha256(stream.read()).hexdigest()
        if actual != PIN or gdb.parameter("non-stop"):
            raise RuntimeError("Engine hash mismatch or non-stop enabled")
        base = symbol_address(SYMBOL)
        # Verify actual call instructions in this exact build, after relocation.
        for offset, target in ((1072, "0x14527ab10"), (1444, "0x145271630"), (1720, "0x145271630")):
            data = memory(base + offset, 5)
            expected = int(target, 16) + base - 0x142eada20
            if data[0] != 0xe8 or base + offset + 5 + int.from_bytes(data[1:], "little", signed=True) != expected:
                raise RuntimeError("Pinned instruction mapping mismatch")
        state = {"owner":owner, "receipt":receipt, "mode":mode, "base":base}
        breaks.append(Entry("*" + hex(base), internal=True))
        breaks.append(WriteCall("*" + hex(base + 1072), internal=True))
        if mode == "write-phase":
            breaks.append(NtWrite("*" + hex(symbol_address("NtWriteFile")), internal=True))
        else:
            offset = 1449 if mode == "after-primary-to-old" else 1725
            breaks.append(RenameReturn("*" + hex(base + offset), internal=True))
        publish(".armed.json", {"pid":inferior.pid,"owner":owner,"mode":mode,
                                "engine":engine,"engine_sha256":actual,"base":hex(base)})
        print("N7_ARMED")
if gdb.parameter("non-stop"):
    raise RuntimeError("N7_DEBUG_ERROR helper requires all-stop before attach")
Arm()
print("N7_HELPER_READY all_stop=true", flush=True)
end
