"""Frida channel selection, independent of process identity authority."""
from dataclasses import dataclass
import frida
from errors import BridgeError
from session_model import TargetIdentity, TransportSpec

@dataclass
class GadgetLease:
    device: object
    session: object
    def close(self):
        self.session.detach()

class GadgetTcpConnector:
    def __init__(self, spec: TransportSpec):
        options = spec.options
        if options.get("host") != "127.0.0.1":
            raise BridgeError("unsupported", "This connector currently supports authenticated loopback Gadget only")
        if type(options.get("port")) is not int or not 0 < options["port"] < 65536:
            raise BridgeError("invalid_input", "Invalid Gadget port")
        if not isinstance(options.get("token"), str) or not options["token"]:
            raise BridgeError("invalid_input", "Missing Gadget credential")
        self.options = options

    def attach(self, target: TargetIdentity):
        device = frida.get_device_manager().add_remote_device(
            self.options["host"] + ":" + str(self.options["port"]), token=self.options["token"])
        return GadgetLease(device, device.attach(target.process.pid, persist_timeout=30))

def resolve_transport(spec):
    if spec.kind != "gadget-tcp":
        raise BridgeError("unsupported", "Transport connector is not implemented: " + spec.kind)
    return GadgetTcpConnector(spec)
