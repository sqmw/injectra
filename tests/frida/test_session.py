import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/"crates/frida/helper"))
from connection import BridgeError, read_session, deadline

class SessionContract(unittest.TestCase):
    def setUp(self):
        self.directory=tempfile.TemporaryDirectory()
        self.path=Path(self.directory.name)/"session.json"
        self.data=dict(schema=1,pid=123,process_start="start",transport="gadget",port=1234,token="test-only")
        self.path.write_text(json.dumps(self.data));self.path.chmod(0o600)
    def tearDown(self):self.directory.cleanup()
    def test_private_regular_file_required(self):
        self.path.chmod(0o644)
        with self.assertRaises(BridgeError):read_session(self.path)
        self.path.chmod(0o600)
        link=self.path.with_name("link");link.symlink_to(self.path)
        with self.assertRaises(BridgeError):read_session(link)
    @patch("connection.process_start",return_value="different")
    def test_reused_pid_is_rejected_before_connect(self,_):
        with self.assertRaises(BridgeError) as error:read_session(self.path)
        self.assertEqual(error.exception.certainty,"NotExecuted")
    @patch("connection.process_start",return_value="start")
    def test_invalid_endpoint_and_transport_rejected(self,_):
        for key,value in [("port",0),("port",65536),("transport","arbitrary"),("schema",9)]:
            changed={**self.data,key:value};self.path.write_text(json.dumps(changed))
            with self.assertRaises(BridgeError):read_session(self.path)
    @patch("connection.process_start",return_value="start")
    def test_private_current_session_is_accepted(self,_):
        self.assertEqual(read_session(self.path)["pid"],123)
class DeadlineContract(unittest.TestCase):
    def test_deadline_cancels_blocking_frida_work(self):
        with deadline(.02) as cancellation:
            time.sleep(.04)
            self.assertTrue(cancellation.is_cancelled)
    def test_finished_scope_cancels_only_its_timer(self):
        with deadline(.03) as cancellation:
            pass
        time.sleep(.05)
        self.assertFalse(cancellation.is_cancelled)

if __name__=="__main__":unittest.main()
