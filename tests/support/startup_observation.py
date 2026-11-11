"""Read-only startup evidence helpers. Never terminate a target or submit a payload."""
import json
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path


def launch_environment(base, private_home, mode):
    env = {**base, "XDG_CONFIG_HOME": str(private_home/".config"),
           "XDG_CACHE_HOME": str(private_home/".cache"),
           "XDG_DATA_HOME": str(private_home/".local/share")}
    if mode == "isolated":
        env.update(HOME=str(private_home), CFFIXED_USER_HOME=str(private_home))
    elif mode == "system":
        env.pop("CFFIXED_USER_HOME", None)
    else:
        raise ValueError("Unknown HOME mode")
    for key in ("ELECTRON_RUN_AS_NODE", "DYLD_INSERT_LIBRARIES"):
        env.pop(key, None)
    return env


def process_snapshot():
    rows = {}
    output = subprocess.check_output(
        ["/bin/ps", "-axo", "pid=,ppid=,pgid=,stat=,lstart=,comm="], text=True)
    for line in output.splitlines():
        fields = line.strip().split(None, 9)
        if len(fields) == 10:
            pid, parent, group = map(int, fields[:3])
            rows[pid] = dict(pid=pid, parent=parent, group=group,
                             state=fields[3], started=" ".join(fields[4:9]), executable=fields[9])
    return rows


def is_live(row):
    return not row.get("state", "").startswith("Z") and row["executable"] != "<defunct>"


class Ownership:
    def __init__(self, baseline, root_pid, executable, profile_argument):
        self.baseline = set(baseline)
        self.root_pid = root_pid
        self.executable = executable
        self.profile_argument = profile_argument
        self.known = {}
        self.reused = set()

    def update(self, rows, arguments):
        for pid, old in list(self.known.items()):
            if pid in rows and rows[pid]["started"] != old["started"]:
                self.reused.add(pid)
        changed = True
        while changed:
            changed = False
            for pid, row in rows.items():
                if pid in self.baseline or pid in self.reused or pid in self.known:
                    continue
                parent = self.known.get(row["parent"])
                active_parent = rows.get(row["parent"])
                parent_matches = (parent is not None and active_parent is not None
                                  and parent["started"] == active_parent["started"]
                                  and row["parent"] not in self.reused)
                exact_profile = (row["executable"] == self.executable
                                 and self.profile_argument in arguments.get(pid, "").split())
                if exact_profile:
                    reason = "exact-test-profile"
                elif row["group"] == self.root_pid and self.root_pid not in self.reused:
                    reason = "owned-process-group"
                elif parent_matches:
                    reason = "observed-parent"
                else:
                    continue
                self.known[pid] = {**row, "ownership": reason}
                changed = True
        return {pid: {**rows[pid], "ownership": self.known[pid]["ownership"],
                      "firstObservedParent": self.known[pid]["parent"],
                      "firstObservedGroup": self.known[pid]["group"]}
                for pid in rows if pid in self.known
                and pid not in self.reused
                and rows[pid]["started"] == self.known[pid]["started"]}


def read_targets(port, timeout=3):
    """Bound the complete request, including slow headers/body, in a controller child."""
    started = time.monotonic()
    try:
        worker = subprocess.run([sys.executable, "-B", str(Path(__file__).resolve()),
                                 str(port), str(timeout)], capture_output=True, text=True,
                                timeout=timeout, check=True)
        result = json.loads(worker.stdout)
    except subprocess.TimeoutExpired:
        result = dict(outcome="timeout", errorType="ProbeDeadlineExceeded")
    except Exception as error:
        result = dict(outcome="probe-error", errorType=type(error).__name__, error=str(error))
    result.update(port=port, startedMonotonic=started, elapsedSeconds=time.monotonic()-started,
                  payloadSubmitted=False)
    return result


def _read_targets(port, timeout):
    started = time.monotonic()
    result = {"port": port, "startedMonotonic": started, "payloadSubmitted": False}
    try:
        # No redirect to a different service; this is only the requested local endpoint.
        class NoRedirect(urllib.request.HTTPRedirectHandler):
            def redirect_request(self, *args, **kwargs):
                return None
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
        try:
            response = opener.open(f"http://127.0.0.1:{port}/json/list", timeout=timeout)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            body = response.read(65537)
            result.update(httpStatus=response.status, truncated=len(body) > 65536,
                          rawBody=body[:65536].decode("utf-8", errors="replace"))
        result["outcome"] = "response"
        try:
            data = json.loads(result["rawBody"])
        except ValueError:
            result["jsonParseFailed"] = True
            return result
        result["rawTargets"] = data
        result["classification"] = []
        if isinstance(data, list):
            for target in data:
                if not isinstance(target, dict):
                    result["classification"].append({"reason": "not-an-object"})
                    continue
                url = target.get("url", "")
                reason = ("non-success-status" if result["httpStatus"] != 200 else
                          "truncated" if result["truncated"] else
                          "non-page" if target.get("type") != "page" else
                          "blank" if url in ("", "about:blank") else
                          "startup-data" if url.startswith("data:") else
                          "devtools" if url.startswith("devtools:") else "page-candidate")
                result["classification"].append({"id": target.get("id"), "reason": reason})
    except Exception as error:
        timeout_error = isinstance(error, (TimeoutError, socket.timeout)) or isinstance(
            getattr(error, "reason", None), (TimeoutError, socket.timeout))
        result.update(outcome="timeout" if timeout_error else "request-error",
                      errorType=type(error).__name__, error=str(error))
    result["elapsedSeconds"] = time.monotonic() - started
    return result


if __name__ == "__main__":
    print(json.dumps(_read_targets(int(sys.argv[1]), float(sys.argv[2]))))
