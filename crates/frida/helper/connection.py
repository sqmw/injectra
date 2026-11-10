"""Authenticated process connection; does not spawn, resume or stop target processes."""
import json
import os
import stat
import subprocess
from pathlib import Path
import frida

class BridgeError(Exception):
    def __init__(self, kind, message, certainty="NotExecuted"):
        super().__init__(message)
        self.kind, self.certainty = kind, certainty

def process_start(pid):
    value = subprocess.check_output(["/bin/ps", "-p", str(pid), "-o", "lstart="], text=True).strip()
    if not value:
        raise BridgeError("target_not_found", "Target process no longer exists")
    return value

def read_session(path):
    path = Path(path)
    info = path.lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
        raise BridgeError("invalid_input", "Session must be a private, user-owned regular file (0600)")
    data = json.loads(path.read_text())
    if data.get("schema") != 1 or not isinstance(data.get("pid"), int) or data["pid"] <= 0:
        raise BridgeError("invalid_input", "Invalid session descriptor")
    if process_start(data["pid"]) != data.get("process_start"):
        raise BridgeError("target_not_found", "Target process identity changed; register a new session")
    if data.get("transport") != "gadget":
        raise BridgeError("unsupported", "This route requires an authenticated Gadget session")
    if not isinstance(data.get("port"), int) or not 0 < data["port"] < 65536 or not data.get("token"):
        raise BridgeError("invalid_input", "Invalid Gadget endpoint")
    return data

class Connection:
    def __init__(self, state):
        self.state = state
        self.device = frida.get_device_manager().add_remote_device(
            "127.0.0.1:" + str(state["port"]), token=state["token"])
        self.session = self.device.attach(state["pid"], persist_timeout=30)
        self.script = None

    def load(self, source):
        self.script = self.session.create_script(source, runtime="qjs")
        self.script.load()
        receipt = self.script.exports_sync.identity()
        if (receipt["pid"] != self.state["pid"] or
                receipt["executable"] != self.state["executable"] or
                receipt["arch"] != self.state["arch"] or
                process_start(self.state["pid"]) != self.state["process_start"]):
            self.script.unload()
            self.script = None
            raise BridgeError("target_not_found", "Authenticated agent identity does not match the session")
        return self.script

    def detach(self):
        if self.script is not None:
            self.script.unload()
            self.script = None
        self.session.detach()
