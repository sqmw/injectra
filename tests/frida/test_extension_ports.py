"""Structural extension proof with test-only providers, without mobile runtime support."""
import unittest
from dataclasses import replace
from types import SimpleNamespace
from unittest.mock import patch
from support import current
from session_model import parse_descriptor, DeviceIdentity
from connection import Connection
from errors import BridgeError

class ExtensionPorts(unittest.TestCase):
    def test_an_injected_provider_owns_identity_checks_without_local_os_lookup(self):
        events=[]
        descriptor=parse_descriptor(current())
        descriptor=replace(descriptor,target=replace(descriptor.target,
            device=DeviceIdentity("test-only-provider","test-only-platform","test-device")))
        class Target:
            def bind(self,target):events.append("bind");return target
            def verify_attached(self,session,target):events.append("verify-device")
            def verify_agent(self,target,receipt):events.append("verify-agent")
        class Script:
            exports_sync=SimpleNamespace(identity=lambda:{"pid":123})
            def load(self):events.append("load")
            def unload(self):events.append("unload")
        class Lease:
            session=SimpleNamespace(create_script=lambda *a,**kw:Script())
            def close(self):events.append("detach")
        class Transport:
            def attach(self,target):events.append("attach");return Lease()
        with patch("target_macos.process_start",side_effect=AssertionError("local PID lookup forbidden")),\
             patch("target_macos.current_device",side_effect=AssertionError("local device lookup forbidden")),\
             patch("transport.frida.get_device_manager",side_effect=AssertionError("real network forbidden")):
            connection=Connection(descriptor,target_provider=Target(),connector=Transport())
            connection.load("test-only agent");connection.detach()
        self.assertEqual(events,["bind","attach","verify-device","load","verify-agent","unload","detach"])
    def test_rejected_device_receipt_closes_transport_before_route_load(self):
        events=[]
        class Target:
            def bind(self,target):return target
            def verify_attached(self,session,target):raise BridgeError("target_not_found","wrong device")
        class Lease:
            session=object()
            def close(self):events.append("detach")
        class Transport:
            def attach(self,target):return Lease()
        with self.assertRaises(BridgeError):
            Connection(parse_descriptor(current()),target_provider=Target(),connector=Transport())
        self.assertEqual(events,["detach"])
if __name__=="__main__":unittest.main()
