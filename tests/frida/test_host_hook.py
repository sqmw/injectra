"""Acceptance checks for host calls, not substitutes for real app execution."""
import copy
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("host_hook_smoke", ROOT / "scripts/host-hook-smoke.py")
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class HostHookVerdict(unittest.TestCase):
    def receipts(self):
        observed = dict(calls=2, completed=2, inFlight=0, wrongReceiver=0, wrongThread=0,
                        installed=True, restored=False, running=True, mainThread=True)
        restored = {**observed, "calls": 3, "completed": 3, "installed": False, "restored": True}
        return [observed, restored, copy.deepcopy(restored)]

    def test_completed_calls_between_observation_and_restore_are_accepted(self):
        probe.validate_observation(*self.receipts())

    def test_late_original_call_failure_is_rejected(self):
        receipts = self.receipts()
        receipts[1]["completed"] = receipts[2]["completed"] = 2
        with self.assertRaises(RuntimeError):
            probe.validate_observation(*receipts)

    def test_zero_calls_cannot_be_reported_as_host_hook_success(self):
        receipts = self.receipts()
        receipts[0].update(calls=0, completed=0)
        with self.assertRaises(RuntimeError):
            probe.validate_observation(*receipts)

    def test_wrong_receiver_thread_unfinished_or_unrestored_hook_is_rejected(self):
        for index, key, value in ((0, "wrongReceiver", 1), (1, "wrongThread", 1),
                                  (1, "inFlight", 1), (2, "restored", False),
                                  (2, "installed", True), (2, "running", False)):
            receipts = self.receipts()
            receipts[index][key] = value
            with self.subTest(index=index, key=key), self.assertRaises(RuntimeError):
                probe.validate_observation(*receipts)

    def test_decreasing_or_continuing_counters_are_rejected(self):
        for restored_count, after_count in ((1, 1), (3, 4)):
            receipts = self.receipts()
            for receipt, count in zip(receipts[1:], (restored_count, after_count)):
                receipt.update(calls=count, completed=count)
            with self.subTest(counts=(restored_count, after_count)), self.assertRaises(RuntimeError):
                probe.validate_observation(*receipts)


if __name__ == "__main__":
    unittest.main()
