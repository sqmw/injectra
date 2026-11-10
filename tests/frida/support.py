"""Data-only fixtures. These are not registered target/device implementations."""
import sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/"crates/frida/helper"))
from session_model import *

def legacy():
    return {"schema":1,"pid":123,"process_start":"start","transport":"gadget","port":1234,
            "token":"test-only","executable":"/fixture","arch":"arm64",
            "routes":["electron-gadget","webkit"],"ack_log":"/test/startup.log"}

def current():
    return {"schema":2,
            "device":{"provider":"macos-local","platform":"macos","instance_id":"boot:test"},
            "process":{"pid":123,"started":"start","executable":"/fixture","arch":"arm64"},
            "connection":{"kind":"gadget-tcp","options":{"host":"127.0.0.1","port":1234,"token":"test-only"}},
            "routes":{"electron-gadget":{"profile":"macos-electron-v8","options":{
                "acknowledgements":{"kind":"controller-file","path":"/test/startup.log"}}},
                "webkit":{"profile":"macos-appkit","options":{}}}}
