"""The only implementation allowed to interpret a PID through local macOS tools."""
from dataclasses import replace
import hashlib
import subprocess
import sys
from pathlib import Path
from errors import BridgeError
from session_model import DeviceIdentity, TargetIdentity

def boot_identity(value):
    if not isinstance(value, str) or not value.strip():
        raise BridgeError("unsupported", "macOS boot identity unavailable")
    return "boot:" + hashlib.sha256(value.strip().lower().encode()).hexdigest()

def process_start(pid):
    result = subprocess.run(["/bin/ps","-p",str(pid),"-o","lstart="],
                            capture_output=True,text=True,timeout=3)
    value = result.stdout.strip()
    if result.returncode or not value:
        raise BridgeError("target_not_found", "Target process no longer exists")
    return value

def current_device():
    if sys.platform != "darwin":
        raise BridgeError("unsupported", "macOS-local target requires a macOS controller on that device")
    result = subprocess.run(["/usr/sbin/sysctl","-n","kern.bootsessionuuid"],
                            capture_output=True,text=True,timeout=3)
    if result.returncode:
        raise BridgeError("unsupported", "Cannot verify macOS device instance")
    return DeviceIdentity("macos-local","macos",boot_identity(result.stdout))

class MacOSLocalTarget:
    def bind(self, target: TargetIdentity):
        expected = target.device
        if expected.provider != "macos-local" or expected.platform != "macos":
            raise BridgeError("unsupported", "This target provider only implements local macOS")
        observed = current_device()
        if expected.instance_id is not None and expected.instance_id != observed.instance_id:
            raise BridgeError("target_not_found", "Device instance changed; register a new session")
        if process_start(target.process.pid) != target.process.started:
            raise BridgeError("target_not_found", "Target process generation changed; register a new session")
        return replace(target, device=observed)

    def probe(self, session):
        script = session.create_script(Path(__file__).with_name("target_macos_identity.js").read_text(),runtime="qjs")
        try:
            script.load()
            return script.exports_sync.identity()
        finally:
            script.unload()

    def verify_attached(self, session, target):
        self.verify_probe(target, self.probe(session))

    def verify_probe(self, target, receipt):
        if receipt.get("platform") != "darwin" or boot_identity(receipt.get("boot")) != target.device.instance_id:
            raise BridgeError("target_not_found", "Authenticated endpoint belongs to a different device instance")
        self.verify_agent(target, receipt)

    def verify_agent(self, target, receipt):
        expected = target.process
        if (receipt.get("pid") != expected.pid or receipt.get("executable") != expected.executable
                or receipt.get("arch") != expected.arch):
            raise BridgeError("target_not_found", "Authenticated target process identity differs from the session")
        self.bind(target)
