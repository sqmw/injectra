"""Versioned target identities and transport descriptions; no OS or Frida calls."""
from dataclasses import dataclass, replace
import hashlib
import json
from typing import Any, Mapping
from errors import BridgeError

def object_field(value, name):
    if not isinstance(value, dict):
        raise BridgeError("invalid_input", "Expected object: " + name)
    return value

def text_field(value, name):
    if not isinstance(value, str) or not value:
        raise BridgeError("invalid_input", "Expected nonempty string: " + name)
    return value

def positive_pid(value):
    if type(value) is not int or value <= 0:
        raise BridgeError("invalid_input", "Invalid target process ID")
    return value

@dataclass(frozen=True)
class DeviceIdentity:
    provider: str
    platform: str
    instance_id: str | None

@dataclass(frozen=True)
class ProcessIdentity:
    """Target-owned metadata; started is an opaque provider generation, not a host timestamp contract."""
    pid: int
    started: str
    executable: str
    arch: str

@dataclass(frozen=True)
class TargetIdentity:
    device: DeviceIdentity
    process: ProcessIdentity

@dataclass(frozen=True)
class TransportSpec:
    kind: str
    options: Mapping[str, Any]

@dataclass(frozen=True)
class RouteBinding:
    profile: str
    options: Mapping[str, Any]

@dataclass(frozen=True)
class SessionDescriptor:
    target: TargetIdentity
    connection: TransportSpec
    routes: Mapping[str, RouteBinding]
    legacy: bool = False

    @property
    def device(self):
        return self.target.device

    @property
    def process(self):
        return self.target.process

    def route(self, name):
        if name not in self.routes:
            raise BridgeError("unsupported", "Session is not registered for the selected backend")
        return self.routes[name]

    def page_id(self, route, local_key):
        if not self.device.instance_id:
            raise BridgeError("invalid_input", "Target device identity has not been verified")
        binding = self.route(route)
        identity = [self.device.provider, self.device.platform, self.device.instance_id,
                    self.process.pid, self.process.started, self.process.executable,
                    self.process.arch, route, binding.profile]
        scope = hashlib.sha256(json.dumps(identity, separators=(",", ":")).encode()).hexdigest()[:32]
        return scope + ":" + str(local_key)

    def to_json(self):
        if not self.device.instance_id:
            raise BridgeError("invalid_input", "Cannot persist an unverified device identity")
        return {
            "schema": 2,
            "device": {"provider": self.device.provider, "platform": self.device.platform,
                       "instance_id": self.device.instance_id},
            "process": {"pid": self.process.pid, "started": self.process.started,
                        "executable": self.process.executable, "arch": self.process.arch},
            "connection": {"kind": self.connection.kind, "options": dict(self.connection.options)},
            "routes": {name: {"profile": value.profile, "options": dict(value.options)}
                       for name, value in self.routes.items()},
        }

def parse_descriptor(data):
    data = object_field(data, "session")
    if type(data.get("schema")) is not int:
        raise BridgeError("invalid_input", "Invalid session version")
    if data["schema"] == 1:
        return parse_legacy(data)
    if data["schema"] != 2:
        raise BridgeError("unsupported", "Unsupported session descriptor version")
    if any(key in data for key in ("pid","process_start","executable","arch","transport","port","token","ack_log")):
        raise BridgeError("invalid_input", "Mixed legacy and v2 session identity fields")
    device = object_field(data.get("device"), "device")
    process = object_field(data.get("process"), "process")
    connection = object_field(data.get("connection"), "connection")
    routes = object_field(data.get("routes"), "routes")
    if not routes:
        raise BridgeError("invalid_input", "No route bindings")
    bindings = {}
    for name, item in routes.items():
        text_field(name, "route name")
        item = object_field(item, "route binding")
        bindings[name] = RouteBinding(text_field(item.get("profile"), "route profile"),
                                      object_field(item.get("options", {}), "route options").copy())
    return SessionDescriptor(
        TargetIdentity(
            DeviceIdentity(text_field(device.get("provider"), "device provider"),
                           text_field(device.get("platform"), "device platform"),
                           text_field(device.get("instance_id"), "device instance")),
            ProcessIdentity(positive_pid(process.get("pid")), text_field(process.get("started"), "process generation"),
                            text_field(process.get("executable"), "target executable"),
                            text_field(process.get("arch"), "target architecture"))),
        TransportSpec(text_field(connection.get("kind"), "connection kind"),
                      object_field(connection.get("options"), "connection options").copy()),
        bindings)

def parse_legacy(data):
    if any(key in data for key in ("device","process","connection")):
        raise BridgeError("invalid_input", "A v1 session cannot declare a different target-device model")
    # v1 was emitted exclusively by the macOS-local registrar. Never infer a remote device.
    if data.get("transport") != "gadget":
        raise BridgeError("unsupported", "Unknown legacy transport")
    names = data.get("routes")
    if not isinstance(names, list) or not names or any(not isinstance(n, str) for n in names):
        raise BridgeError("invalid_input", "Invalid legacy routes")
    profiles = {"electron-gadget": "macos-electron-v8", "webkit": "macos-appkit"}
    routes = {}
    for name in names:
        if name not in profiles:
            raise BridgeError("unsupported", "Unknown legacy host route")
        options = {}
        if name == "electron-gadget":
            options = {"acknowledgements": {"kind": "controller-file",
                       "path": text_field(data.get("ack_log"), "controller acknowledgement path")}}
        routes[name] = RouteBinding(profiles[name], options)
    return SessionDescriptor(
        TargetIdentity(DeviceIdentity("macos-local", "macos", None),
            ProcessIdentity(positive_pid(data.get("pid")), text_field(data.get("process_start"), "process generation"),
                            text_field(data.get("executable"), "target executable"),
                            text_field(data.get("arch"), "target architecture"))),
        TransportSpec("gadget-tcp", {"host": "127.0.0.1", "port": data.get("port"), "token": data.get("token")}),
        routes, legacy=True)
