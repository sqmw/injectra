"""Explicit macOS owned-app native Hook test, separate from Gadget/Web validation."""
import argparse
import hashlib
import json
import os
import plistlib
import subprocess
import sys
import tempfile
import math
from pathlib import Path
import frida

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "crates/frida/helper"))
from deadline import deadline


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--deadline", type=float, default=15,
                        help="Seconds per attach/RPC phase; use a smaller value for deadline tests")
    args = parser.parse_args()
    if sys.platform != "darwin" or not math.isfinite(args.deadline) or args.deadline <= 0:
        raise RuntimeError("Requires macOS and a positive deadline")
    app = args.app.resolve()
    info = plistlib.loads((app / "Contents/Info.plist").read_bytes())
    executable = (app / "Contents/MacOS" / info["CFBundleExecutable"]).resolve()
    active = subprocess.check_output(["/bin/ps", "-axo", "comm="], text=True).splitlines()
    if str(executable) in [p.strip() for p in active]:
        raise RuntimeError("Target is already running; no user instance was stopped")
    subprocess.run(["/usr/bin/codesign", "--verify", "--deep", "--strict", str(app)], check=True)
    fingerprint = hashlib.sha256(executable.read_bytes()).hexdigest()
    base = Path(os.environ.get("INJECTRA_CACHE_DIR", Path(os.environ.get("XDG_CACHE_HOME", Path.home()/".cache"))/"injectra"))
    (base / "runs").mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix="douyin-native-hook-", dir=base / "runs"))
    run.chmod(0o700)
    home = run / "home"
    (home / "Library/Application Support").mkdir(parents=True)
    profile = home / "profile"
    env = {**os.environ, "HOME": str(home), "CFFIXED_USER_HOME": str(home),
           "XDG_CONFIG_HOME": str(home/".config"), "XDG_CACHE_HOME": str(home/".cache"),
           "XDG_DATA_HOME": str(home/".local/share")}
    for key in ("DYLD_INSERT_LIBRARIES", "ELECTRON_RUN_AS_NODE"):
        env.pop(key, None)
    result = {"passed": False, "route": "external-attach-interceptor", "app": str(app),
              "version": info["CFBundleShortVersionString"], "frida": frida.__version__,
              "run": str(run), "profile": str(profile), "executable_sha256_before": fingerprint,
              "deadlineSeconds": args.deadline, "stage": "launch"}
    child = None
    session = None
    script = None
    try:
        with (run / "startup.log").open("w") as log:
            child = subprocess.Popen([str(executable), "--user-data-dir="+str(profile), "--no-first-run"],
                                     env=env, stdout=log, stderr=log, start_new_session=True)
        result["pid"] = child.pid
        print(json.dumps({"run": str(run), "pid": child.pid, "stage": "attach"}), flush=True)
        result["stage"] = "attach"
        with deadline(args.deadline):
            session = frida.get_local_device().attach(child.pid)
        result["stage"] = "agent-load"
        with deadline(args.deadline):
            script = session.create_script((ROOT / "tests/frida/native-hook.js").read_text(), runtime="qjs")
            script.load()
            identity = script.exports_sync.identity()
        result["identity"] = identity
        if identity["pid"] != child.pid or identity["executable"] != str(executable):
            raise RuntimeError("Owned target identity mismatch")
        result["stage"] = "interceptor"
        with deadline(args.deadline):
            receipt = script.exports_sync.verify()
        result["hook"] = receipt
        assert receipt["calls"] == [{"input": -7, "result": 7}, {"input": -13, "result": 13}], receipt
        assert (receipt["first"], receipt["second"], receipt["after"]) == (7, 13, 19), receipt
        assert receipt["detached"] and receipt["countAfterDetach"] == receipt["countAfterFinalCall"] == 2, receipt
        result["passed"] = True
    except Exception as error:
        result["failure"] = {"type": type(error).__name__, "message": str(error)}
    finally:
        try:
            with deadline(5):
                if script is not None:
                    script.unload()
                if session is not None:
                    session.detach()
        except Exception as error:
            result["cleanupFailure"] = str(error)
            result["passed"] = False
        if child is not None:
            if child.poll() is None:
                child.terminate()
                try:
                    child.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    child.kill()
                    child.wait(timeout=5)
            result["ownedProcessExited"] = child.poll() is not None
            result["exitCode"] = child.returncode
        result["executable_sha256_after"] = hashlib.sha256(executable.read_bytes()).hexdigest()
        result["executableUnchanged"] = result["executable_sha256_after"] == fingerprint
        signed = subprocess.run(["/usr/bin/codesign", "--verify", "--deep", "--strict", str(app)], capture_output=True, text=True)
        result["signatureValidAfter"] = signed.returncode == 0
        if not result["executableUnchanged"] or not result["signatureValidAfter"]:
            result["passed"] = False
        (run / "result.json").write_text(json.dumps(result, indent=2)+"\n")
    print(json.dumps(result), flush=True)
    return 0 if result["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
