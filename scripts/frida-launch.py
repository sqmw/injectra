"""Compatibility CLI wrapper over crates/frida/helper/session_ops.py."""
import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "crates/frida/helper"))
from errors import BridgeError  # noqa: E402
from session_ops import BOOTSTRAP, launch_session  # noqa: E402


def launch(app, gadget, arguments, route, launcher="direct"):
    if sys.platform != "darwin":
        raise RuntimeError("This launcher only supports macOS-local .app targets")
    try:
        result = launch_session(
            app,
            gadget,
            arguments,
            route,
            launcher,
            emit_progress=True,
            terminate_on_failure=False,
        )
        return {"session": result["session"], "pid": result["pid"], "route": result["route"]}
    except BridgeError as error:
        if error.kind == "timeout":
            raise TimeoutError(str(error)) from error
        raise RuntimeError(str(error)) from error


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--app", type=Path, required=True)
    p.add_argument("--gadget", type=Path, required=True)
    p.add_argument("--argument", action="append", default=[])
    p.add_argument("--route", choices=["electron-gadget", "webkit"], default="electron-gadget")
    p.add_argument("--launcher", choices=["direct", "launchservices"], default="direct")
    a = p.parse_args()
    print(json.dumps(launch(a.app.resolve(), a.gadget.resolve(), a.argument, a.route, a.launcher)), flush=True)
