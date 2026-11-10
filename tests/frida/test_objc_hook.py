"""Verdict and ownership gates; these tests do not prove native compatibility."""
import importlib.util
import plistlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("objc_hook_smoke", ROOT / "scripts/objc-hook-smoke.py")
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class ObjcHookVerdict(unittest.TestCase):
    def receipt(self):
        return dict(baseline="123", first="123", second="123", after="123",
                    mechanism="objc-imp-swap", hookedCalls=2, callsAfterRestore=2,
                    restored=True, rootMethodUnchanged=True)

    def test_complete_receipt_and_incomplete_execution(self):
        probe.verify_receipt(self.receipt())
        for key in self.receipt():
            receipt = self.receipt()
            del receipt[key]
            with self.subTest(missing=key), self.assertRaises(RuntimeError):
                probe.verify_receipt(receipt)

    def test_hook_absent_still_installed_or_mutating_cannot_pass(self):
        for key, value in (("hookedCalls", 0), ("callsAfterRestore", 3), ("first", "999"),
                           ("after", "999"), ("restored", False), ("rootMethodUnchanged", False)):
            with self.subTest(key=key), self.assertRaises(RuntimeError):
                probe.verify_receipt({**self.receipt(), key: value})

    def test_existing_user_instance_is_rejected_before_launch(self):
        with tempfile.TemporaryDirectory() as folder:
            app = Path(folder) / "fixture.app"
            (app / "Contents").mkdir(parents=True)
            (app / "Contents/Info.plist").write_bytes(plistlib.dumps({"CFBundleExecutable": "fixture"}))
            executable = (app / "Contents/MacOS/fixture").resolve()
            with (patch.object(probe.sys, "platform", "darwin"),
                  patch.object(probe.sys, "argv", ["probe", "--app", str(app), "--gadget", "unused"]),
                  patch.object(probe.subprocess, "check_output", return_value=str(executable)+"\n"),
                  patch.object(probe.subprocess, "Popen") as launch,
                  patch.object(probe.subprocess, "run") as command):
                with self.assertRaisesRegex(RuntimeError, "already running"):
                    probe.main()
                launch.assert_not_called()
                command.assert_not_called()


if __name__ == "__main__":
    unittest.main()
