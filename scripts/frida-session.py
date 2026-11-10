"""Register an existing authenticated macOS-local Gadget process; never starts an application."""
import argparse
from dataclasses import replace
import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"crates/frida/helper"))
from deadline import deadline
from errors import BridgeError
from session_store import private_json, write_session
from session_model import (DeviceIdentity, ProcessIdentity, TargetIdentity, TransportSpec,
                           RouteBinding, SessionDescriptor, positive_pid, text_field)
from target_macos import MacOSLocalTarget, current_device, process_start
from transport import resolve_transport

def register(state_path, output, route="electron-gadget"):
    old = private_json(state_path)
    pid = positive_pid(old.get("pid"))
    executable = text_field(old.get("executable"), "target executable")
    profiles = {"electron-gadget":"macos-electron-v8", "webkit":"macos-appkit"}
    if route not in profiles:
        raise BridgeError("unsupported", "This registrar has no such host profile")
    options = {}
    if route == "electron-gadget":
        ack = state_path.parent/"startup.log"
        if not ack.is_file():
            raise BridgeError("invalid_input", "Node acknowledgement log is missing")
        options = {"acknowledgements":{"kind":"controller-file","path":str(ack.resolve())}}
    spec = TransportSpec("gadget-tcp",{"host":"127.0.0.1","port":old.get("port"),"token":old.get("token")})
    connector = resolve_transport(spec)
    provider = MacOSLocalTarget()
    target = TargetIdentity(current_device(),ProcessIdentity(pid,process_start(pid),executable,"unverified"))
    target = provider.bind(target)
    lease = None
    try:
        with deadline(10):
            lease = connector.attach(target)
            receipt = provider.probe(lease.session)
            process = replace(target.process,arch=text_field(receipt.get("arch"),"target architecture"))
            target = replace(target,process=process)
            provider.verify_probe(target,receipt)
            descriptor = SessionDescriptor(target,spec,{route:RouteBinding(profiles[route],options)})
            write_session(output,descriptor)
            return {"session":str(output),"pid":pid,"route":route}
    finally:
        with deadline(3):
            if lease is not None:lease.close()

if __name__=="__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state",required=True,type=Path)
    parser.add_argument("--output",required=True,type=Path)
    parser.add_argument("--route",choices=["electron-gadget","webkit"],default="electron-gadget")
    args=parser.parse_args()
    import json
    print(json.dumps(register(args.state,args.output,args.route)))
