"""Compatibility CLI wrapper over crates/frida/helper/session_ops.py."""
import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "crates/frida/helper"))
from session_ops import register_session  # noqa: E402


def register(state_path, output, route="electron-gadget"):
    return register_session(state_path, output, route)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--route", choices=["electron-gadget", "webkit"], default="electron-gadget")
    args = parser.parse_args()
    print(json.dumps(register(args.state, args.output, args.route)))
