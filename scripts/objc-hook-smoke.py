"""Optional macOS Gadget-required Objective-C dispatch test on an owned app instance."""
import argparse
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tests/frida"))
import gadget_probe


def verify_receipt(receipt):
    values = [receipt.get(key) for key in ("baseline", "first", "second", "after")]
    if not isinstance(values[0], str) or len(set(values)) != 1:
        raise RuntimeError("Original method result was not preserved")
    if (receipt.get("mechanism") != "objc-imp-swap"
            or receipt.get("hookedCalls") != 2 or receipt.get("callsAfterRestore") != 2
            or receipt.get("restored") is not True or receipt.get("rootMethodUnchanged") is not True):
        raise RuntimeError("Hook callbacks, restoration, or superclass isolation failed")


def exercise(script, result, timeout):
    result["stage"] = "objc-hook"
    with gadget_probe.deadline(timeout):
        receipt = script.exports_sync.verify()
    result["hook"] = receipt
    verify_receipt(receipt)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--gadget", type=Path, required=True)
    parser.add_argument("--deadline", type=float, default=15,
                        help="Seconds per listener/Frida phase; 2 for fast deadline diagnostics")
    args = parser.parse_args()
    if sys.platform != "darwin" or not math.isfinite(args.deadline) or args.deadline <= 0:
        parser.error("Requires macOS and a finite positive deadline")
    return gadget_probe.run_owned(args, agent_path=ROOT / "tests/frida/objc-dispatch-hook.js",
                                  route="gadget-required-objc-imp-swap", prefix="douyin-objc-hook-",
                                  exercise=exercise)


if __name__ == "__main__":
    raise SystemExit(main())
