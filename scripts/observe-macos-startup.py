"""Observe an explicitly owned app launch. No payload and no automatic subtree cleanup."""
import argparse
import concurrent.futures
import hashlib
import json
import os
import plistlib
import socket
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tests/support"))
from startup_observation import Ownership, process_snapshot, read_targets, launch_environment, is_live


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--mode", choices=["cdp", "plain"], default="cdp")
    parser.add_argument("--home-mode", choices=["isolated", "system"], default="isolated",
                        help="Keep normal HOME for system services; Chromium profile stays temporary")
    parser.add_argument("--seconds", type=int, default=40)
    args = parser.parse_args()
    if sys.platform != "darwin" or not 5 <= args.seconds <= 120:
        parser.error("Requires macOS and seconds between 5 and 120")
    app = args.app.resolve()
    info = plistlib.loads((app / "Contents/Info.plist").read_bytes())
    executable = (app / "Contents/MacOS" / info["CFBundleExecutable"]).resolve()
    if not executable.is_relative_to(app):
        raise RuntimeError("Executable outside selected app")
    baseline = process_snapshot()
    if any(row["executable"] == str(executable) for row in baseline.values()):
        raise RuntimeError("Existing instance: normal exit must be explicitly authorized first")
    cache = Path(os.environ.get("INJECTRA_CACHE_DIR",
                 Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache")) / "injectra"))
    (cache / "runs").mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix="startup-observe-", dir=cache / "runs")); run.chmod(0o700)
    home = run / "home"; (home / "Library/Application Support").mkdir(parents=True)
    profile_argument = "--user-data-dir=" + str(home / "profile")
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0)); port = reservation.getsockname()[1]
    command = [str(executable), profile_argument, "--disk-cache-dir=" + str(home / "chromium-cache")]
    if args.mode == "cdp":
        command += ["--remote-debugging-port=" + str(port), "--remote-debugging-address=127.0.0.1"]
    command += ["--no-first-run"]
    env = launch_environment(os.environ, home, args.home_mode)
    report = dict(app=str(app), version=info["CFBundleShortVersionString"], mode=args.mode,
                  seconds=args.seconds, command=command, profile=str(home / "profile"),
                  homeMode=args.home_mode, configuredHome=env.get("HOME"),
                  payloadSubmitted=False, automaticChildCleanup=False,
                  port=port if args.mode == "cdp" else None,
                  mainSha256Before=hashlib.sha256(executable.read_bytes()).hexdigest(),
                  sources={str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
                           for path in [Path(__file__), ROOT / "tests/support/startup_observation.py"]})
    (run / "request.json").write_text(json.dumps(report, indent=2) + "\n")
    with (run / "startup.log").open("w") as log:
        child = subprocess.Popen(command, env=env, stdout=log, stderr=log, start_new_session=True)
    report["rootPid"] = child.pid
    tracker = Ownership(baseline, child.pid, str(executable), profile_argument)
    print(json.dumps({"run": str(run), "pid": child.pid, "mode": args.mode, "port": port}), flush=True)
    start = time.monotonic(); pending = None; pending_context = None
    last_key = None; last_http = -10; last_print = start
    report["observationStartMonotonic"] = start
    responses = 0; root_exit_seen = False
    with concurrent.futures.ThreadPoolExecutor(max_workers=1) as executor, \
            (run / "events.jsonl").open("w") as events, (run / "http.jsonl").open("w") as http:
        while time.monotonic() - start < args.seconds:
            now = time.monotonic(); rows = process_snapshot(); arguments = {}
            for pid, row in rows.items():
                if pid not in baseline and row["executable"] == str(executable):
                    arguments[pid] = subprocess.run(["/bin/ps", "-ww", "-p", str(pid), "-o", "args="],
                                                   capture_output=True, text=True).stdout.strip()
            present = tracker.update(rows, arguments)
            live = {pid: row for pid, row in present.items() if is_live(row)}
            code = child.poll()
            if code is not None and not root_exit_seen:
                report["rootExitObservedAt"] = now - start; root_exit_seen = True
                print(json.dumps({"event": "launcher-exited", "code": code,
                                  "continueObserving": True}), flush=True)
            listener = subprocess.run(["lsof", "-nP", "-iTCP:" + str(port), "-sTCP:LISTEN", "-Fpn"],
                                      capture_output=True, text=True).stdout if args.mode == "cdp" else ""
            owners = [int(line[1:]) for line in listener.splitlines() if line.startswith("p")]
            loopback = all(line.startswith(("n127.0.0.1:", "n[::1]:"))
                           for line in listener.splitlines() if line.startswith("n"))
            snapshot = dict(liveProcesses=list(live.values()), presentProcesses=list(present.values()),
                            mainArguments=arguments,
                            listener=listener, listenerOwners=owners, rootExitCode=code)
            key = json.dumps(snapshot, sort_keys=True)
            if key != last_key:
                events.write(json.dumps({"elapsed": now-start, **snapshot}) + "\n"); events.flush(); last_key = key
            if pending is not None and pending.done():
                result = pending.result()
                # If completion raced this snapshot, retain the future until the next poll.
                if now >= result["startedMonotonic"] + result["elapsedSeconds"]:
                    result["listenerBefore"] = pending_context
                    result["ownershipSnapshotStartedMonotonic"] = now
                    result["listenerOwnershipConfirmedAfter"] = (
                        set(owners) == set(pending_context)
                        and all(pid in live and live[pid]["started"] == old["started"]
                                for pid, old in pending_context.items()))
                    http.write(json.dumps(result) + "\n"); http.flush()
                    responses += 1; pending = None
            if args.mode == "cdp" and owners and loopback and all(pid in live for pid in owners):
                if pending is None and now - last_http >= 1:
                    pending_context = {pid: live[pid] for pid in owners}
                    pending = executor.submit(read_targets, port); last_http = now
            if now - last_print >= 10:
                print(json.dumps({"elapsed": round(now-start), "trackedLive": len(live),
                                  "rootExit": code, "httpReceipts": responses}), flush=True); last_print = now
            time.sleep(.25)
        if pending is not None:
            result = pending.result(); result["listenerBefore"] = pending_context
            result["listenerOwnershipConfirmedAfter"] = False
            result["ownershipNote"] = "Observation ended before another ownership snapshot"
            http.write(json.dumps(result) + "\n"); responses += 1
    report.update(rootExitCode=child.poll(), liveProcesses=list(live.values()),
                  presentProcesses=list(present.values()),
                  httpReceipts=responses, mainSha256After=hashlib.sha256(executable.read_bytes()).hexdigest(),
                  cleanup="Not performed; controller must verify exact identities before normal exit")
    (run / "result.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"run": str(run), "rootExit": child.returncode,
                      "livePids": list(live), "httpReceipts": responses, "payloadSubmitted": False}), flush=True)


if __name__ == "__main__":
    main()
