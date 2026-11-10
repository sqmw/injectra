"""Import an explicitly selected existing Gadget state into a private product session."""
import argparse
import json
import os
import stat
import subprocess
from pathlib import Path
import frida

IDENTITY = "rpc.exports={identity(){return {pid:Process.id,arch:Process.arch,executable:Process.mainModule.path};}};"
def private_read(path):
    info = path.lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
        raise ValueError("Input must be a private user-owned regular file")
    return json.loads(path.read_text())

def register(state_path, output, route="electron-gadget"):
    old = private_read(state_path)
    pid, port = old["pid"], old["port"]
    if not isinstance(pid, int) or pid <= 0 or not isinstance(port, int) or not 0 < port < 65536:
        raise ValueError("Invalid process or endpoint")
    start = subprocess.check_output(["/bin/ps","-p",str(pid),"-o","lstart="],text=True).strip()
    device = frida.get_device_manager().add_remote_device("127.0.0.1:"+str(port),token=old["token"])
    session = device.attach(pid)
    script = session.create_script(IDENTITY)
    try:
        script.load()
        receipt = script.exports_sync.identity()
        if receipt["pid"] != pid or receipt["executable"] != old["executable"]:
            raise ValueError("Authenticated process differs from the requested state")
        if start != subprocess.check_output(["/bin/ps","-p",str(pid),"-o","lstart="],text=True).strip():
            raise ValueError("Process changed during registration")
        ack = state_path.parent/"startup.log"
        if route == "electron-gadget" and not ack.is_file():
            raise ValueError("Node acknowledgement log is missing")
        data = dict(schema=1,transport="gadget",routes=[route],pid=pid,port=port,
            token=old["token"],executable=receipt["executable"],arch=receipt["arch"],
            process_start=start,ack_log=str(ack.resolve()))
        output.parent.mkdir(parents=True,exist_ok=True)
        fd = os.open(output,os.O_CREAT|os.O_EXCL|os.O_WRONLY,0o600)
        with os.fdopen(fd,"w") as stream:
            json.dump(data,stream,indent=2)
        return {"session":str(output),"pid":pid,"route":route}
    finally:
        script.unload()
        session.detach()

if __name__ == "__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state",required=True,type=Path)
    parser.add_argument("--output",required=True,type=Path)
    parser.add_argument("--route",choices=["electron-gadget","webkit"],default="electron-gadget")
    args=parser.parse_args()
    print(json.dumps(register(args.state,args.output,args.route)))
