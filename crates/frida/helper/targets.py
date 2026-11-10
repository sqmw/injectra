"""Explicit target-provider registry. Unsupported devices never fall back to local PID lookup."""
from errors import BridgeError
from target_macos import MacOSLocalTarget

def resolve_target(device):
    if device.provider == "macos-local" and device.platform == "macos":
        return MacOSLocalTarget()
    raise BridgeError("unsupported", "Target provider is not implemented for " + device.provider + "/" + device.platform)
