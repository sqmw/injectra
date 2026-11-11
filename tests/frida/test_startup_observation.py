"""Protect launch evidence against handoff, PID reuse, filtering and HTTP delay."""
import json
import sys
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "support"))
from startup_observation import Ownership, read_targets, launch_environment, is_live


def row(pid, parent=1, group=1, started="first", executable="/fixture/app"):
    return dict(pid=pid, parent=parent, group=group, started=started, executable=executable)


class StartupObservation(unittest.TestCase):
    def test_zombie_is_present_but_not_live(self):
        self.assertFalse(is_live({**row(10), "state": "Zs"}))
        self.assertFalse(is_live({**row(10), "executable": "<defunct>"}))
        self.assertTrue(is_live({**row(10), "state": "Ss"}))

    def test_system_home_preserves_parent_context_without_changing_it(self):
        original = dict(HOME="/user", CFFIXED_USER_HOME="/old-test",
                        DYLD_INSERT_LIBRARIES="/old-agent", PATH="/bin")
        system = launch_environment(original, Path("/new-test/home"), "system")
        self.assertEqual(system["HOME"], "/user")
        self.assertEqual(system["XDG_CACHE_HOME"], "/new-test/home/.cache")
        self.assertNotIn("CFFIXED_USER_HOME", system)
        self.assertNotIn("DYLD_INSERT_LIBRARIES", system)
        self.assertEqual(original["CFFIXED_USER_HOME"], "/old-test")
        isolated = launch_environment(original, Path("/new-test/home"), "isolated")
        self.assertEqual(isolated["HOME"], "/new-test/home")
        self.assertEqual(isolated["CFFIXED_USER_HOME"], "/new-test/home")

    def test_handoff_survives_launcher_exit(self):
        tracker = Ownership({}, 10, "/fixture/app", "--user-data-dir=/owned/profile")
        tracker.update({10: row(10, group=10), 11: row(11, parent=10, group=10)}, {})
        live = tracker.update({11: row(11, group=10)}, {})
        self.assertEqual(set(live), {11})
        self.assertEqual(live[11]["parent"], 1)
        self.assertEqual(live[11]["firstObservedParent"], 10)

    def test_foreign_instance_and_reused_pid_do_not_gain_ownership(self):
        tracker = Ownership({9: row(9)}, 10, "/fixture/app", "--user-data-dir=/owned/profile")
        live = tracker.update({9: row(9), 10: row(10, group=10), 20: row(20)}, {})
        self.assertEqual(set(live), {10})
        live = tracker.update({10: row(10, group=10, started="replacement")}, {})
        self.assertEqual(live, {})

    def test_reparented_exact_profile_is_distinguished_from_similar_prefix(self):
        tracker = Ownership({}, 10, "/fixture/app", "--user-data-dir=/owned/profile")
        rows = {11: row(11), 12: row(12)}
        args = {11: "/fixture/app --user-data-dir=/owned/profile",
                12: "/fixture/app --user-data-dir=/owned/profile-other"}
        self.assertEqual(set(tracker.update(rows, args)), {11})

    def test_raw_targets_and_timeout_remain_distinct(self):
        class Handler(BaseHTTPRequestHandler):
            delay = 0
            status = 200
            drip = False
            def do_GET(self):
                time.sleep(self.delay)
                body = json.dumps([{"id": "early", "type": "page", "url": "about:blank"}]).encode()
                try:
                    self.send_response(self.status); self.end_headers()
                    if self.drip:
                        for byte in body:
                            self.wfile.write(bytes([byte])); self.wfile.flush(); time.sleep(.03)
                    else:
                        self.wfile.write(body)
                except (BrokenPipeError, ConnectionResetError):
                    pass
            def log_message(self, *args):
                pass
        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True); thread.start()
        try:
            result = read_targets(server.server_port)
            self.assertEqual(result["outcome"], "response")
            self.assertEqual(result["rawTargets"][0]["id"], "early")
            self.assertEqual(result["classification"][0]["reason"], "blank")
            Handler.status = 403
            result = read_targets(server.server_port)
            self.assertEqual(result["httpStatus"], 403)
            self.assertIn("rawBody", result)
            self.assertEqual(result["classification"][0]["reason"], "non-success-status")
            Handler.delay = .15
            result = read_targets(server.server_port, timeout=.03)
            self.assertEqual(result["outcome"], "timeout")
            self.assertNotIn("rawTargets", result)
            Handler.delay = 0; Handler.drip = True
            started = time.monotonic()
            result = read_targets(server.server_port, timeout=.3)
            self.assertEqual(result["outcome"], "timeout")
            self.assertLess(time.monotonic()-started, 1.5)
        finally:
            server.shutdown(); server.server_close(); thread.join()


if __name__ == "__main__":
    unittest.main()
