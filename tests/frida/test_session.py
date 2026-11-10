import json
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch
from support import legacy, current
from session_model import parse_descriptor
from session_store import read_session, write_session
from errors import BridgeError
from deadline import deadline

class SessionContract(unittest.TestCase):
    def setUp(self):
        self.directory=tempfile.TemporaryDirectory()
        self.path=Path(self.directory.name)/"session.json"
        self.data=legacy()
        self.path.write_text(json.dumps(self.data));self.path.chmod(0o600)
    def tearDown(self):self.directory.cleanup()
    def test_private_regular_file_required(self):
        self.path.chmod(0o644)
        with self.assertRaises(BridgeError):read_session(self.path)
        self.path.chmod(0o600)
        link=self.path.with_name("link");link.symlink_to(self.path)
        with self.assertRaises(BridgeError):read_session(link)
    def test_legacy_read_does_not_inspect_host_or_rewrite_file(self):
        before=self.path.read_bytes()
        with patch("subprocess.run") as run:
            session=read_session(self.path)
            run.assert_not_called()
        self.assertEqual(self.path.read_bytes(),before)
        self.assertTrue(session.legacy)
        self.assertEqual(session.device.provider,"macos-local")
        self.assertIsNone(session.device.instance_id)
        self.assertEqual(session.process.pid,123)
        self.assertEqual(session.route("electron-gadget").options["acknowledgements"]["path"],"/test/startup.log")
        with self.assertRaises(BridgeError):session.page_id("webkit","view")
    def test_new_files_roundtrip_v2_without_overwriting(self):
        session=parse_descriptor(current())
        path=self.path.with_name("v2.json")
        write_session(path,session)
        self.assertEqual(json.loads(path.read_text())["schema"],2)
        self.assertEqual(path.stat().st_mode & 0o777,0o600)
        self.assertEqual(read_session(path).page_id("webkit","view"),session.page_id("webkit","view"))
        with self.assertRaises(FileExistsError):write_session(path,session)
    def test_malformed_private_json_is_an_input_failure_before_target_work(self):
        self.path.write_text("{not-json")
        with patch("subprocess.run") as run,self.assertRaises(BridgeError) as error:
            read_session(self.path)
        self.assertEqual(error.exception.kind,"invalid_input")
        run.assert_not_called()
    def test_duplicate_fields_cannot_override_declared_device_identity(self):
        self.path.write_text('{"schema":2,"device":{"platform":"android"},"device":{"platform":"macos"}}')
        with patch("subprocess.run") as run,self.assertRaises(BridgeError) as error:
            read_session(self.path)
        self.assertEqual(error.exception.kind,"invalid_input")
        run.assert_not_called()
    def test_mixed_versions_cannot_hide_a_device_override(self):
        data=legacy();data["device"]={"provider":"mobile","platform":"android"}
        with patch("subprocess.run") as run,self.assertRaises(BridgeError):parse_descriptor(data)
        run.assert_not_called()
        data=current();data["pid"]=456
        with self.assertRaises(BridgeError):parse_descriptor(data)
    def test_target_path_and_generation_remain_provider_owned_metadata(self):
        data=current();data["process"].update(executable="/device-only/app",started="opaque-device-generation")
        with patch("pathlib.Path.exists",side_effect=AssertionError("no host path lookup")):
            result=parse_descriptor(data)
        self.assertEqual(result.process.executable,"/device-only/app")
        self.assertEqual(result.process.started,"opaque-device-generation")
    def test_unrecognized_schema_and_malformed_identity_fail_without_io(self):
        bad=[[],{"schema":True},{**legacy(),"schema":9},{**legacy(),"pid":True},
             {**legacy(),"routes":["ios"]},{**legacy(),"process_start":""}]
        for value in bad:
            with self.subTest(value=value):
                with self.assertRaises(BridgeError):parse_descriptor(value)

class DeadlineContract(unittest.TestCase):
    def test_deadline_cancels_blocking_frida_work(self):
        with deadline(.02) as cancellation:
            time.sleep(.04)
            self.assertTrue(cancellation.is_cancelled)
    def test_finished_scope_cancels_only_its_timer(self):
        with deadline(.03) as cancellation:pass
        time.sleep(.05)
        self.assertFalse(cancellation.is_cancelled)
if __name__=="__main__":unittest.main()
