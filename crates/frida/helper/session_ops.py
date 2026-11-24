"""Controller-side Frida Gadget session launch, registration and verification."""
from dataclasses import replace
import json
import os
import plistlib
import secrets
import shutil
import socket
import subprocess
import sys
import time
from pathlib import Path

import frida

from connection import Connection
from deadline import deadline
from errors import BridgeError
from session_model import (
    ProcessIdentity,
    RouteBinding,
    SessionDescriptor,
    TargetIdentity,
    TransportSpec,
    positive_pid,
    text_field,
)
from session_store import private_json, read_session, write_session
from target_macos import MacOSLocalTarget, current_device, process_start
from transport import resolve_transport

BOOTSTRAP = """const env=n=>{const p=new NativeFunction(Module.getGlobalExportByName('getenv'),'pointer',['pointer'])(Memory.allocUtf8String(n));return p.isNull()?null:p.readUtf8String();};
rpc.exports={bootstrap(){const home=env('HOME');let foundationHome=null;const homeFn=Module.findGlobalExportByName('NSHomeDirectory');if(homeFn!==null){const object=new NativeFunction(homeFn,'pointer',[])();const selector=new NativeFunction(Module.getGlobalExportByName('sel_registerName'),'pointer',['pointer'])(Memory.allocUtf8String('UTF8String'));const chars=new NativeFunction(Module.getGlobalExportByName('objc_msgSend'),'pointer',['pointer','pointer'])(object,selector);foundationHome=chars.isNull()?null:chars.readUtf8String();}const unset=new NativeFunction(Module.getGlobalExportByName('unsetenv'),'int',['pointer']);unset(Memory.allocUtf8String('DYLD_INSERT_LIBRARIES'));return {pid:Process.id,executable:Process.mainModule.path,home,foundationHome,dyld:env('DYLD_INSERT_LIBRARIES')};}};"""

IDENTITY_PROBE = (
    "rpc.exports={identity(){return {pid:Process.id,arch:Process.arch,"
    "executable:Process.mainModule.path};}};"
)


def _write_private_new(path: Path, text: str) -> None:
    fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as stream:
        stream.write(text)


def _write_private_existing(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")
    os.chmod(path, 0o600)


def _cache_dir() -> Path:
    return Path(
        os.environ.get(
            "INJECTRA_CACHE_DIR",
            Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache")) / "injectra",
        )
    )


def register_session(state_path, output, route="electron-gadget"):
    state_path = Path(state_path)
    output = Path(output)
    if output.exists():
        raise BridgeError("invalid_input", "Session output file already exists")
    old = private_json(state_path)
    pid = positive_pid(old.get("pid"))
    executable = text_field(old.get("executable"), "target executable")
    profiles = {"electron-gadget": "macos-electron-v8", "webkit": "macos-appkit"}
    if route not in profiles:
        raise BridgeError("unsupported", "This registrar has no such host profile")
    options = {}
    if route == "electron-gadget":
        ack = state_path.parent / "startup.log"
        if not ack.is_file():
            raise BridgeError("invalid_input", "Node acknowledgement log is missing")
        options = {"acknowledgements": {"kind": "controller-file", "path": str(ack.resolve())}}
    spec = TransportSpec(
        "gadget-tcp",
        {"host": "127.0.0.1", "port": old.get("port"), "token": old.get("token")},
    )
    connector = resolve_transport(spec)
    provider = MacOSLocalTarget()
    target = TargetIdentity(
        current_device(),
        ProcessIdentity(pid, process_start(pid), executable, "unverified"),
    )
    target = provider.bind(target)
    lease = None
    try:
        with deadline(10):
            lease = connector.attach(target)
            receipt = provider.probe(lease.session)
            process = replace(
                target.process,
                arch=text_field(receipt.get("arch"), "target architecture"),
            )
            target = replace(target, process=process)
            provider.verify_probe(target, receipt)
            descriptor = SessionDescriptor(target, spec, {route: RouteBinding(profiles[route], options)})
            try:
                write_session(output, descriptor)
            except FileExistsError as error:
                raise BridgeError("invalid_input", "Session output file already exists") from error
            return {"session": str(output), "pid": pid, "route": route}
    finally:
        with deadline(3):
            if lease is not None:
                lease.close()


def launch_session(
    app,
    gadget,
    arguments,
    route="electron-gadget",
    launcher="direct",
    output=None,
    *,
    emit_progress=False,
    terminate_on_failure=True,
):
    if sys.platform != "darwin":
        raise BridgeError("unsupported", "This launcher only supports macOS-local .app targets")
    if route not in ("electron-gadget", "webkit"):
        raise BridgeError("unsupported", "Unsupported session route")
    if launcher not in ("direct", "launchservices"):
        raise BridgeError("invalid_input", "Unsupported launcher mode")
    app = Path(app).resolve()
    gadget = Path(gadget).resolve()
    if not app.is_dir() or not (app / "Contents/Info.plist").is_file():
        raise BridgeError("invalid_input", "Expected a valid macOS .app bundle")
    if not gadget.is_file():
        raise BridgeError("invalid_input", "Gadget library not found")
    out_session = Path(output).resolve() if output is not None else None
    if out_session is not None and out_session.exists():
        raise BridgeError("invalid_input", "Session output file already exists")

    info = plistlib.loads((app / "Contents/Info.plist").read_bytes())
    exe_name = info.get("CFBundleExecutable")
    if not isinstance(exe_name, str) or not exe_name or "/" in exe_name or exe_name in (".", ".."):
        raise BridgeError("invalid_input", "Invalid CFBundleExecutable in Info.plist")
    executable = (app / "Contents/MacOS" / exe_name).resolve()
    if not executable.is_file() or not executable.is_relative_to(app):
        raise BridgeError("invalid_input", "Bundle executable escapes bundle or is missing")

    processes = subprocess.check_output(["/bin/ps", "-axo", "comm="], text=True).splitlines()
    if str(executable) in [p.strip() for p in processes]:
        raise BridgeError(
            "connection",
            "Application already running; no instance was stopped or restarted",
        )

    base = _cache_dir()
    run = base / "frida/runs" / ("launch-" + str(time.time_ns()))
    run.mkdir(parents=True, mode=0o700)
    os.chmod(run, 0o700)
    home = run / "home"
    (home / "Library/Application Support").mkdir(parents=True)
    library = run / "FridaGadget.dylib"
    shutil.copy2(gadget, library)

    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        port = sock.getsockname()[1]
    token = secrets.token_hex(24)
    config = {
        "interaction": {
            "type": "listen",
            "address": "127.0.0.1",
            "port": port,
            "token": token,
            "on_load": "wait",
            "on_port_conflict": "fail",
        },
        "runtime": "qjs",
        "code_signing": "required",
    }
    _write_private_new(run / "FridaGadget.config", json.dumps(config))

    log = run / "startup.log"
    env = {
        **os.environ,
        "HOME": str(home),
        "CFFIXED_USER_HOME": str(home),
        "DYLD_INSERT_LIBRARIES": str(library),
    }
    env.pop("ELECTRON_RUN_AS_NODE", None)
    for key, relative in (
        ("XDG_CONFIG_HOME", ".config"),
        ("XDG_CACHE_HOME", ".cache"),
        ("XDG_DATA_HOME", ".local/share"),
    ):
        env[key] = str(home / relative)

    args = list(arguments)
    if route == "electron-gadget":
        args = ["--user-data-dir=" + str(home / "profile"), "--no-first-run", *args]
    if launcher == "direct":
        command = [str(executable), *args]
    else:
        command = [
            "/usr/bin/open",
            "-n",
            "-a",
            str(app),
            "--env",
            "HOME=" + str(home),
            "--env",
            "CFFIXED_USER_HOME=" + str(home),
            "--env",
            "DYLD_INSERT_LIBRARIES=" + str(library),
            "--stdout",
            str(log),
            "--stderr",
            str(log),
            "--args",
            *args,
        ]

    with log.open("w") as output_stream:
        child = subprocess.Popen(
            command,
            env=env,
            stdout=output_stream,
            stderr=output_stream,
            start_new_session=True,
        )

    state = {
        "pid": None,
        "port": port,
        "token": token,
        "executable": str(executable),
        "launcher_pid": child.pid,
        "phase": "launching",
    }
    statefile = run / "state.json"
    _write_private_new(statefile, json.dumps(state))
    if emit_progress:
        print(
            json.dumps({"run": str(run), "launcher_pid": child.pid, "phase": "launching"}),
            flush=True,
        )

    session = None
    script = None
    try:
        for _ in range(150):
            if child.poll() is not None and (launcher == "direct" or child.returncode != 0):
                raise BridgeError("connection", "Owned launcher exited; inspect " + str(log))
            try:
                with socket.create_connection(("127.0.0.1", port), timeout=0.1):
                    break
            except OSError:
                time.sleep(0.1)
        else:
            raise BridgeError("timeout", "Gadget listener unavailable; inspect " + str(log))

        with deadline(10):
            device = frida.get_device_manager().add_remote_device(
                "127.0.0.1:" + str(port), token=state["token"]
            )
            pid = device.spawn(["re.frida.Gadget"])
            state.update(pid=pid, phase="verifying")
            _write_private_existing(statefile, json.dumps(state))
            session = device.attach(pid)
            script = session.create_script(BOOTSTRAP)
            script.load()
            receipt = script.exports_sync.bootstrap()
            if receipt != {
                "pid": pid,
                "executable": str(executable),
                "home": str(home),
                "foundationHome": str(home),
                "dyld": None,
            }:
                raise BridgeError("connection", "Bootstrap identity or inheritance check failed")
            if launcher == "direct" and pid != child.pid:
                raise BridgeError("connection", "Unexpected process identity")
            device.resume(pid)
            state.update(phase="running")
            _write_private_existing(statefile, json.dumps(state))
    except Exception:
        state["phase"] = "failed"
        _write_private_existing(statefile, json.dumps(state))
        if terminate_on_failure and launcher == "direct" and child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=2)
            except subprocess.TimeoutExpired:
                pass
        raise
    finally:
        with deadline(3):
            if script is not None:
                script.unload()
            if session is not None:
                session.detach()

    descriptor_path = out_session if out_session is not None else (run / "session.json")
    try:
        registered = register_session(statefile, descriptor_path, route)
    except Exception:
        if terminate_on_failure and launcher == "direct" and child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=2)
            except subprocess.TimeoutExpired:
                pass
        raise
    registered["state"] = str(statefile)
    registered["run"] = str(run)
    registered["launcher_pid"] = child.pid
    return registered


def verify_session(session_path, route=None):
    state = read_session(session_path)
    if route is not None:
        state.route(route)
    conn = None
    try:
        with deadline(10):
            conn = Connection(state)
            conn.load(IDENTITY_PROBE)
    finally:
        with deadline(5):
            if conn is not None:
                conn.detach()
    return {
        "valid": True,
        "schema": 1 if state.legacy else 2,
        "legacy": state.legacy,
        "device": {
            "provider": state.device.provider,
            "platform": state.device.platform,
        },
        "process": {
            "pid": state.process.pid,
            "arch": state.process.arch,
            "executable": state.process.executable,
        },
        "routes": sorted(state.routes.keys()),
    }


def _dispatch(request):
    if not isinstance(request, dict) or request.get("schema") != 1:
        raise BridgeError("invalid_input", "Invalid session request schema")
    op = request.get("operation")
    payload = request.get("payload")
    if not isinstance(payload, dict):
        raise BridgeError("invalid_input", "Missing session request payload")
    if op == "launch":
        return launch_session(
            payload["app"],
            payload["gadget"],
            payload.get("arguments", []),
            payload.get("route", "electron-gadget"),
            payload.get("launcher", "direct"),
            payload.get("output"),
            emit_progress=False,
            terminate_on_failure=True,
        )
    if op == "register":
        return register_session(
            payload["state"],
            payload["output"],
            payload.get("route", "electron-gadget"),
        )
    if op == "verify":
        return verify_session(payload["session"], payload.get("route"))
    raise BridgeError("invalid_input", "Unsupported session operation")


if __name__ == "__main__":
    try:
        req = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
        result = {"schema": 1, "ok": True, "value": _dispatch(req)}
    except frida.OperationCancelledError:
        result = {
            "schema": 1,
            "ok": False,
            "error": {
                "kind": "timeout",
                "certainty": "NotExecuted",
                "message": "Frida session operation deadline exceeded",
            },
        }
    except BridgeError as error:
        result = {
            "schema": 1,
            "ok": False,
            "error": {
                "kind": error.kind,
                "certainty": "NotExecuted",
                "message": str(error),
            },
        }
    except Exception as error:
        result = {
            "schema": 1,
            "ok": False,
            "error": {
                "kind": "connection",
                "certainty": "NotExecuted",
                "message": type(error).__name__ + ": " + str(error),
            },
        }
    print(json.dumps(result, ensure_ascii=False), flush=True)
