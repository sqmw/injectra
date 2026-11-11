"""Regenerate Electron >=41 packaging metadata in an owned test copy; keep enforcement on.

Format: electron/packager src/mac.ts at 21c5955cc3877e93d268ceaae7b1beae673aced7.
"""
from pathlib import Path
import hashlib
import json
import plistlib
import sys

app = Path(sys.argv[1]).resolve()
marker = app.parent / "ownership.json"
if app.name != "Injectra-Test.app" or not app.parent.name.startswith("resource-patch-") or not marker.is_file():
    raise RuntimeError("Not an owned test copy")
ownership = json.loads(marker.read_text())
if ownership.get("app") != str(app) or ownership.get("createdBy") != "T-038/E-001":
    raise RuntimeError("Ownership mismatch")
source = Path(ownership["source"]).resolve()
if source == app:
    raise RuntimeError("Original must be preserved")

def digest(info):
    value = hashlib.sha256()
    for key, entry in sorted(info["ElectronAsarIntegrity"].items()):
        for text in (key, entry["algorithm"], entry["hash"]):
            value.update(text.encode("utf-8"))
    return value.digest()

before = digest(plistlib.loads((source / "Contents/Info.plist").read_bytes()))
after = digest(plistlib.loads((app / "Contents/Info.plist").read_bytes()))
framework = (app / "Contents/Frameworks/Electron Framework.framework/Electron Framework").resolve()
if not framework.is_relative_to(app):
    raise RuntimeError("Framework outside test copy")
data = bytearray(framework.read_bytes())
sentinel = b"AGbevlPCksUGKNL8TSn7wGmJEuJsXb2A"
positions = []
cursor = 0
while True:
    offset = data.find(sentinel, cursor)
    if offset < 0:
        break
    start = offset + len(sentinel)
    if start + 34 > len(data) or data[start:start + 2] != b"\x01\x01" or data[start + 2:start + 34] != before:
        raise RuntimeError("Unexpected existing integrity slot; no write performed")
    positions.append(start)
    cursor = start + 34
if not positions:
    raise RuntimeError("No integrity digest slot; no write performed")
for start in positions:
    data[start:start + 34] = b"\x01\x01" + after
framework.write_bytes(data)
print(json.dumps({"slots": positions, "enforcementRetained": True, "before": before.hex(), "after": after.hex()}))
