"""Shared owned-process transport for optional Gadget-required Hook experiments."""
import hashlib
import importlib.util
import json
import math
import os
import plistlib
import secrets
import shutil
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import frida

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "crates/frida/helper"))
from deadline import deadline

# Reuse the production launcher's identity/home/inheritance check, not its Web route.
spec = importlib.util.spec_from_file_location("gadget_launcher", ROOT / "scripts/frida-launch.py")
launcher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(launcher)


def stop_owned(child):
    if child.poll() is None:
        child.terminate()
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=5)


def run_owned(args, *, agent_path, route, prefix, exercise):
    if sys.platform != "darwin" or not math.isfinite(args.deadline) or args.deadline <= 0:
        raise ValueError("Requires macOS and a finite positive deadline")
    app = args.app.resolve()
    info = plistlib.loads((app / "Contents/Info.plist").read_bytes())
    executable = (app / "Contents/MacOS" / info["CFBundleExecutable"]).resolve()
    active = subprocess.check_output(["/bin/ps", "-axo", "comm="], text=True).splitlines()
    if str(executable) in [line.strip() for line in active]:
        raise RuntimeError("Application already running; no existing instance was stopped")
    subprocess.run(["codesign", "--verify", "--deep", "--strict", str(app)], check=True)
    fingerprint = hashlib.sha256(executable.read_bytes()).hexdigest()
    base = Path(os.environ.get("INJECTRA_CACHE_DIR",
                Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache")) / "injectra"))
    (base / "runs").mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix=prefix, dir=base / "runs"))
    run.chmod(0o700)
    home = run / "home"
    (home / "Library/Application Support").mkdir(parents=True)
    library = run / "FridaGadget.dylib"
    shutil.copy2(args.gadget.resolve(), library)
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]
    token = secrets.token_hex(24)
    config = {"interaction": {"type": "listen", "address": "127.0.0.1", "port": port,
              "token": token, "on_load": "wait", "on_port_conflict": "fail"},
              "runtime": "qjs", "code_signing": "required"}
    config_path = run / "FridaGadget.config"
    config_path.write_text(json.dumps(config))
    config_path.chmod(0o600)
    env = {**os.environ, "HOME": str(home), "CFFIXED_USER_HOME": str(home),
           "DYLD_INSERT_LIBRARIES": str(library), "XDG_CONFIG_HOME": str(home / ".config"),
           "XDG_CACHE_HOME": str(home / ".cache"), "XDG_DATA_HOME": str(home / ".local/share")}
    env.pop("ELECTRON_RUN_AS_NODE", None)
    agent = agent_path.read_text()
    result = {"passed": False, "route": route, "run": str(run),
              "app": str(app), "version": info["CFBundleShortVersionString"],
              "frida": frida.__version__, "stage": "launch", "deadlineSeconds": args.deadline,
              "agentSha256": hashlib.sha256(agent.encode()).hexdigest(),
              "gadgetSha256": hashlib.sha256(library.read_bytes()).hexdigest(),
              "executableSha256Before": fingerprint}
    child = session = script = None
    try:
        with (run / "startup.log").open("w") as log:
            child = subprocess.Popen([str(executable), "--user-data-dir=" + str(home / "profile"),
                                      "--no-first-run"], env=env, stdout=log, stderr=log,
                                     start_new_session=True)
        result["pid"] = child.pid
        print(json.dumps({"run": str(run), "pid": child.pid}), flush=True)
        result["stage"] = "listener"
        end = time.monotonic() + args.deadline
        while True:
            if child.poll() is not None:
                raise RuntimeError("Owned process exited before listener")
            try:
                with socket.create_connection(("127.0.0.1", port), timeout=.1):
                    break
            except OSError:
                if time.monotonic() >= end:
                    raise TimeoutError("Gadget listener unavailable")
                time.sleep(.1)
        result["stage"] = "bootstrap"
        with deadline(args.deadline):
            device = frida.get_device_manager().add_remote_device("127.0.0.1:" + str(port), token=token)
            pid = device.spawn(["re.frida.Gadget"])
            if pid != child.pid:
                raise RuntimeError("Unexpected Gadget PID")
            session = device.attach(pid)
            script = session.create_script(launcher.BOOTSTRAP, runtime="qjs")
            script.load()
            identity = script.exports_sync.bootstrap()
            if identity != {"pid": pid, "executable": str(executable), "home": str(home),
                            "foundationHome": str(home), "dyld": None}:
                raise RuntimeError("Bootstrap identity/home/inheritance mismatch")
            result["bootstrap"] = identity
            device.resume(pid)
            script.unload()
            script = None
        result["stage"] = "agent-load"
        with deadline(args.deadline):
            script = session.create_script(agent, runtime="qjs")
            script.load()
            identity = script.exports_sync.identity()
            result["identity"] = identity
            if (identity["pid"] != pid or identity["executable"] != str(executable)
                    or identity["policy"] != "required"):
                raise RuntimeError("Hook target identity/policy mismatch")
        exercise(script, result, args.deadline)
        if child.poll() is not None:
            raise RuntimeError("Owned process exited before agent cleanup")
        result["passed"] = True
    except Exception as error:
        result["failure"] = {"type": type(error).__name__, "message": str(error)}
    finally:
        # On unknown restoration, exit our process before unloading callback code.
        if child is not None and not result["passed"]:
            stop_owned(child)
        try:
            with deadline(5):
                if script is not None and child is not None and child.poll() is None:
                    script.unload()
                    result["agentUnloaded"] = True
                if session is not None and not session.is_detached:
                    session.detach()
                if result["passed"] and (not result.get("agentUnloaded") or child.poll() is not None):
                    raise RuntimeError("Live process/agent unload was not confirmed")
        except Exception as error:
            result["cleanupFailure"] = str(error)
            result["passed"] = False
        if child is not None:
            stop_owned(child)
            result["ownedProcessExited"] = child.poll() is not None
            result["exitCode"] = child.returncode
        result["executableSha256After"] = hashlib.sha256(executable.read_bytes()).hexdigest()
        result["executableUnchanged"] = result["executableSha256After"] == fingerprint
        check = subprocess.run(["codesign", "--verify", "--deep", "--strict", str(app)],
                               capture_output=True, text=True)
        result["signatureValidAfter"] = check.returncode == 0
        if not result["executableUnchanged"] or not result["signatureValidAfter"]:
            result["passed"] = False
        (run / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result), flush=True)
    return 0 if result["passed"] else 1
