"""Real owned-process handoff test; the HTTP fixture is not an app compatibility proof."""
import json
import os
import plistlib
import shlex
import signal
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SERVER = '''import json,os,sys
from pathlib import Path
from http.server import BaseHTTPRequestHandler,HTTPServer
pidfile=Path(sys.argv[1]);pidfile.write_text(str(os.getpid()))
port=int(next(a.split('=',1)[1] for a in sys.argv if a.startswith('--remote-debugging-port=')))
class Handler(BaseHTTPRequestHandler):
 def do_GET(self):
  self.send_response(200);self.end_headers()
  self.wfile.write(json.dumps([{'id':'startup-only','type':'page','url':'about:blank'}]).encode())
 def log_message(self,*args):pass
HTTPServer(('127.0.0.1',port),Handler).serve_forever()
'''


@unittest.skipUnless(sys.platform == "darwin", "macOS process-table observer")
class RealHandoff(unittest.TestCase):
    def test_observation_continues_after_launcher_exits_and_retains_raw_targets(self):
        cache = Path(os.environ.get("INJECTRA_CACHE_DIR",
                     Path(os.environ.get("XDG_CACHE_HOME", Path.home()/".cache"))/"injectra"))
        (cache / "runs").mkdir(parents=True, exist_ok=True)
        run = Path(tempfile.mkdtemp(prefix="startup-observer-fixture-", dir=cache/"runs"))
        app = run / "Fixture.app"; macos = app / "Contents/MacOS"; macos.mkdir(parents=True)
        (app / "Contents/Info.plist").write_bytes(plistlib.dumps(
            {"CFBundleExecutable": "fixture", "CFBundleShortVersionString": "fixture"}))
        server = run / "server.py"; server.write_text(SERVER)
        pidfile = run / "server.pid"
        exe = macos / "fixture"
        exe.write_text("#!/bin/sh\n" + shlex.join([sys.executable, "-B", str(server), str(pidfile)])
                       + ' "$@" &\nexit 0\n')
        exe.chmod(0o700)
        try:
            result = subprocess.run([sys.executable, "-B", str(ROOT/"scripts/observe-macos-startup.py"),
                                     "--app", str(app), "--seconds", "5"],
                                    cwd=ROOT, capture_output=True, text=True, timeout=15)
            (run / "observer.log").write_text(result.stdout + result.stderr)
            self.assertEqual(result.returncode, 0, result.stderr)
            summary = json.loads(result.stdout.splitlines()[-1])
            report_dir = Path(summary["run"])
            report = json.loads((report_dir/"result.json").read_text())
            receipts = [json.loads(line) for line in (report_dir/"http.jsonl").read_text().splitlines()]
            self.assertEqual(report["rootExitCode"], 0)
            self.assertFalse(report["automaticChildCleanup"])
            self.assertTrue(report["liveProcesses"])
            self.assertTrue(any(r.get("rawTargets", [{}])[0].get("id") == "startup-only"
                                and r.get("listenerOwnershipConfirmedAfter") for r in receipts))
            self.assertTrue(all(not r["payloadSubmitted"] for r in receipts))
        finally:
            if pidfile.exists():
                pid = int(pidfile.read_text())
                args = subprocess.run(["/bin/ps", "-ww", "-p", str(pid), "-o", "args="],
                                      capture_output=True, text=True)
                if args.returncode == 0 and str(server) in args.stdout and str(pidfile) in args.stdout:
                    os.kill(pid, signal.SIGTERM)  # Only this generated HTTP fixture.


if __name__ == "__main__":
    unittest.main()
