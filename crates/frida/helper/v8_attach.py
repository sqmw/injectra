"""Explicit-target Windows MSVC V8 dispatch helper for injectra-frida."""
import json
import ntpath
import pathlib
import sys
import time

from target_windows import identity


def reply_ok(value):
    print(json.dumps({"schema": 1, "ok": True, "value": value}))


def reply_err(kind, certainty, message):
    print(
        json.dumps(
            {
                "schema": 1,
                "ok": False,
                "error": {
                    "kind": kind,
                    "certainty": certainty,
                    "message": str(message),
                },
            }
        )
    )


def main():
    try:
        request = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
    except Exception as error:
        reply_err("invalid_input", "NotExecuted", f"Invalid request: {error}")
        return
    if request.get("schema") != 1 or request.get("operation") != "attach-v8":
        reply_err("protocol", "NotExecuted", "Unsupported v8_attach operation")
        return
    if sys.platform != "win32":
        reply_err("unsupported", "NotExecuted", "Windows V8 attach requires Windows")
        return

    payload = request.get("payload") or {}
    pid = payload.get("pid")
    exe = payload.get("exe")
    started = payload.get("started")
    module = payload.get("module") or exe
    source = payload.get("source")
    if not isinstance(pid, int) or pid <= 0 or not isinstance(exe, str) or not isinstance(source, str):
        reply_err("invalid_input", "NotExecuted", "pid, exe, and source are required")
        return

    expected_exe = ntpath.normcase(ntpath.normpath(exe))
    try:
        actual = identity(pid)
    except Exception as error:
        reply_err("target_not_found", "NotExecuted", f"Cannot verify target PID {pid}: {error}")
        return

    if actual["exe"] != expected_exe:
        reply_err(
            "target_not_found",
            "NotExecuted",
            f"Target executable mismatch for PID {pid}: expected {expected_exe}, found {actual['exe']}",
        )
        return
    if started is not None and actual["started"] != int(started):
        reply_err(
            "target_not_found",
            "NotExecuted",
            f"Target generation mismatch for PID {pid}",
        )
        return

    try:
        import frida
    except Exception as error:
        reply_err("unsupported", "NotExecuted", f"Frida Python package unavailable: {error}")
        return

    session = None
    script = None
    events = []
    certainty = "NotExecuted"
    agent_info = None
    detach_info = None
    try:
        session = frida.attach(pid)
        recheck = identity(pid)
        if recheck != actual:
            reply_err(
                "target_not_found",
                "NotExecuted",
                f"Target generation changed during attach for PID {pid}",
            )
            return
        agent_source = pathlib.Path(__file__).with_name("v8_windows.js").read_text(
            encoding="utf-8"
        )
        script = session.create_script(agent_source, runtime="qjs")
        script.on("message", lambda msg, _data: events.append(msg))
        script.load()
        agent_info = script.exports_sync.start(module)
        certainty = "Unknown"
        script.exports_sync.execute("injectra-v8", source)
        for _ in range(150):
            if any(
                m.get("payload", {}).get("event") in ("v8-run", "v8-error")
                or m.get("type") == "error"
                for m in events
            ):
                break
            time.sleep(0.1)
    except Exception as error:
        err_kind = "connection" if certainty == "NotExecuted" else "execution"
        err_msg = str(error)
        if "Windows x86/x64 only" in err_msg or "Explicit engine module" in err_msg or "unable to find export" in err_msg.lower():
            err_kind = "unsupported"
        if script is not None:
            try:
                detach_info = script.exports_sync.stop()
                if certainty == "Unknown" and detach_info.get("attempted") == 0:
                    certainty = "NotExecuted"
            except Exception:
                pass
        if session is not None:
            try:
                session.detach()
            except Exception:
                pass
        reply_err(err_kind, certainty, err_msg)
        return
    finally:
        if script is not None and detach_info is None:
            try:
                detach_info = script.exports_sync.stop()
            except Exception:
                pass
        if session is not None:
            try:
                session.detach()
            except Exception:
                pass

    run_event = next(
        (m.get("payload") for m in events if m.get("payload", {}).get("event") == "v8-run"),
        None,
    )
    err_event = next(
        (m.get("payload") for m in events if m.get("payload", {}).get("event") == "v8-error"),
        None,
    )
    script_err = next((m for m in events if m.get("type") == "error"), None)

    if run_event and run_event.get("returned"):
        reply_ok(
            {
                "pid": pid,
                "exe": actual["exe"],
                "started": actual["started"],
                "module": module,
                "dispatched": True,
                "certainty": "Executed",
                "agent": agent_info,
                "detach": detach_info,
            }
        )
        return
    if err_event:
        reply_err("execution", "Executed", err_event.get("message", "V8 script error"))
        return
    if run_event and not run_event.get("returned"):
        reply_err("execution", "Executed", "V8 script threw an uncaught exception")
        return
    if script_err:
        reply_err(
            "execution",
            "Unknown",
            script_err.get("description") or script_err.get("stack") or "Frida agent error",
        )
        return
    if detach_info and detach_info.get("attempted") == 0:
        reply_err(
            "timeout",
            "NotExecuted",
            "V8 event pump did not trigger before deadline; zero dispatch attempts confirmed",
        )
        return
    reply_err("timeout", "Unknown", "Timed out waiting for V8 dispatch result")


if __name__ == "__main__":
    main()
