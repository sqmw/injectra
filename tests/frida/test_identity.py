import copy
import importlib.util
import unittest
from dataclasses import replace
from unittest.mock import patch
from support import ROOT, current, legacy
from session_model import parse_descriptor, DeviceIdentity
from target_macos import MacOSLocalTarget, boot_identity
from connection import Connection
from errors import BridgeError

class IdentityScope(unittest.TestCase):
    def test_same_local_page_key_cannot_alias_across_devices_processes_or_routes(self):
        original=current();base=parse_descriptor(original).page_id("webkit","same-view")
        for section,key,value in [("device","instance_id","other-device"),("device","provider","other-provider"),
            ("process","started","new-generation"),("process","pid",456),("process","executable","/other")]:
            data=copy.deepcopy(original);data[section][key]=value
            self.assertNotEqual(parse_descriptor(data).page_id("webkit","same-view"),base)
        self.assertNotEqual(parse_descriptor(original).page_id("electron-gadget","same-view"),base)
    def test_transport_and_controller_files_are_not_target_identity(self):
        data=current();base=parse_descriptor(data).page_id("electron-gadget","same-view")
        data["connection"]["options"].update(port=9876,token="rotated-test-only")
        data["routes"]["electron-gadget"]["options"]["acknowledgements"]["path"]="/another-controller-file"
        self.assertEqual(parse_descriptor(data).page_id("electron-gadget","same-view"),base)
    def test_legacy_and_v2_bind_to_the_same_verified_identity(self):
        v1=parse_descriptor(legacy());v2=parse_descriptor(current())
        device=v2.device
        with patch("target_macos.current_device",return_value=device),patch("target_macos.process_start",return_value="start"):
            bound=replace(v1,target=MacOSLocalTarget().bind(v1.target))
        self.assertEqual(bound.page_id("webkit","page"),v2.page_id("webkit","page"))
        self.assertIsNone(v1.device.instance_id)

class PayloadScopeRejection(unittest.TestCase):
    def test_foreign_or_unscoped_ids_are_rejected_before_payload_dispatch(self):
        for name in ("webkit", "electron-gadget"):
            path=ROOT/("crates/"+("webkit" if name=="webkit" else "electron-gadget")+"/helper/route.py")
            spec=importlib.util.spec_from_file_location(name.replace("-","_")+"_scope_test",path)
            module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
            state=parse_descriptor(current())
            other=replace(state,target=replace(state.target,device=replace(state.device,instance_id="other-device")))
            route=module.Route(None,state,path.parent)
            route.pages=lambda:[{"id":state.page_id(name,"same-page")}]
            from unittest.mock import Mock
            dispatch=Mock(side_effect=AssertionError("payload must not be submitted"))
            if name=="webkit":
                from types import SimpleNamespace
                route.script=SimpleNamespace(exports_sync=SimpleNamespace(evaluate=dispatch))
            else:route.node=dispatch
            for key in (other.page_id(name,"same-page"),"same-page"):
                with self.assertRaises(BridgeError) as error:
                    route.request({"operation":"evaluate","payload":{"target":key,"source":"1+1"}})
                self.assertEqual(error.exception.certainty,"NotExecuted")
            dispatch.assert_not_called()

class TargetValidation(unittest.TestCase):
    def test_unknown_mobile_provider_never_falls_back_to_local_ps_or_transport(self):
        for platform in ("android","ios"):
            data=current();data["device"].update(provider="mobile-test",platform=platform)
            with patch("connection.resolve_transport") as transport,patch("target_macos.process_start") as ps:
                with self.assertRaises(BridgeError) as e:Connection(parse_descriptor(data))
                self.assertEqual(e.exception.kind,"unsupported")
                ps.assert_not_called();transport.assert_not_called()
    def test_unknown_connector_is_rejected_before_local_target_inspection(self):
        data=current();data["connection"]={"kind":"unimplemented-usb","options":{}}
        with patch("target_macos.current_device") as device,patch("target_macos.process_start") as ps:
            with self.assertRaises(BridgeError) as e:Connection(parse_descriptor(data))
            self.assertEqual(e.exception.kind,"unsupported")
            device.assert_not_called();ps.assert_not_called()
    def test_invalid_endpoint_is_rejected_before_process_lookup(self):
        for port in (0,65536,True):
            data=current();data["connection"]["options"]["port"]=port
            with patch("target_macos.process_start") as ps:
                with self.assertRaises(BridgeError):Connection(parse_descriptor(data))
                ps.assert_not_called()
    def test_wrong_boot_or_process_generation_is_rejected(self):
        target=parse_descriptor(current()).target
        with patch("target_macos.current_device",return_value=DeviceIdentity("macos-local","macos","other")):
            with self.assertRaises(BridgeError):MacOSLocalTarget().bind(target)
        with patch("target_macos.current_device",return_value=target.device),patch("target_macos.process_start",return_value="new"):
            with self.assertRaises(BridgeError) as e:MacOSLocalTarget().bind(target)
            self.assertEqual(e.exception.certainty,"NotExecuted")
    def test_authenticated_receipt_must_match_device_and_process(self):
        data=current();data["device"]["instance_id"]=boot_identity("test-boot")
        target=parse_descriptor(data).target
        receipt=dict(platform="darwin",boot="test-boot",pid=123,executable="/fixture",arch="arm64")
        with patch("target_macos.current_device",return_value=target.device),patch("target_macos.process_start",return_value="start"):
            MacOSLocalTarget().verify_probe(target,receipt)
            for key,value in (("boot","other-boot"),("pid",456),("arch","x64"),("executable","/other"),("platform","linux")):
                with self.subTest(key=key),self.assertRaises(BridgeError):
                    MacOSLocalTarget().verify_probe(target,{**receipt,key:value})
    def test_unsupported_host_profile_is_rejected_before_connection(self):
        path=ROOT/"crates/webkit/helper/route.py"
        spec=importlib.util.spec_from_file_location("webkit_route_test",path)
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        data=current();data["routes"]["webkit"]["profile"]="ios-uikit"
        with self.assertRaises(BridgeError) as e:module.Route.validate_binding(parse_descriptor(data))
        self.assertEqual(e.exception.kind,"unsupported")
if __name__=="__main__":unittest.main()
