"""Sign an owned test copy; optional local-library entitlement, existing flags retained."""
import argparse
import json
import plistlib
import subprocess
import tempfile
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("app", type=Path)
parser.add_argument("--outer", action="store_true")
parser.add_argument("--local-libraries", action="store_true", help="Test-copy process entitlements for local ad-hoc library loading")
args = parser.parse_args()
app = args.app.resolve()
legacy = app.name == "aDrive-Test.app" and app.parent.name.startswith("adrive-resource-")
batch = app.name == "Injectra-Test.app" and app.parent.name.startswith("resource-patch-")
if not (legacy or batch):
    raise RuntimeError("Not an owned test copy")
marker = app.parent / "ownership.json"
if not marker.is_file():
    raise RuntimeError("Missing test ownership marker")
ownership = json.loads(marker.read_text())
if (ownership.get("app") != str(app) or ownership.get("createdBy") not in (("T-035/E-001",) if legacy else ("T-038/E-001", "T-039/E-001"))
        or not ownership.get("source") or Path(ownership["source"]).resolve() == app):
    raise RuntimeError("Test ownership marker does not match selected copy")
if ownership.get("dataPolicy") == "shared" and Path(ownership["source"]).name != "QQ.app":
    raise RuntimeError("Shared-data preview is only configured for QQ")
for item in app.rglob("*"):
    if item.is_symlink() and not item.resolve().is_relative_to(app):
        raise RuntimeError("Test copy contains a symlink outside its ownership boundary")

def sign(path):
    command = ["/usr/bin/codesign", "--force", "--sign", "-", "--timestamp=none"]
    temporary = None
    if args.local_libraries and (path.suffix == ".app" or path.parent.name == "MacOS"):
        original = subprocess.run(["/usr/bin/codesign", "-d", "--entitlements", ":-", str(path)], capture_output=True)
        entitlements = plistlib.loads(original.stdout) if original.stdout.strip() else {}
        if batch and ownership.get("dataPolicy") != "shared":
            for key in ("com.apple.security.application-groups", "keychain-access-groups", "com.apple.application-identifier", "com.apple.developer.team-identifier", "com.apple.developer.associated-domains", "com.apple.security.get-task-allow", "get-task-allow"):
                entitlements.pop(key, None)
        entitlements["com.apple.security.cs.disable-library-validation"] = True
        temporary = tempfile.NamedTemporaryFile(dir=app.parent, prefix="test-entitlements-", suffix=".plist")
        temporary.write(plistlib.dumps(entitlements)); temporary.flush()
        command += ["--preserve-metadata=identifier,flags", "--entitlements", temporary.name]
    else:
        command += ["--preserve-metadata=identifier,entitlements,flags"]
    result = subprocess.run([*command, str(path)], capture_output=True, text=True)
    if temporary:
        temporary.close()
    if result.returncode:
        raise RuntimeError(f"Signing failed: {path.relative_to(app)}: {result.stderr}")

if args.outer:
    sign(app)
    print(json.dumps({"signed": "outer", "preservedFlags": True, "localLibraryEntitlement": args.local_libraries}))
else:
    signed = []
    magic = {bytes.fromhex(value) for value in ("feedface", "feedfacf", "cefaedfe", "cffaedfe", "cafebabe", "bebafeca", "cafebabf", "bfbafeca")}
    for path in sorted(app.rglob("*"), key=lambda path: len(path.parts), reverse=True):
        if path.is_symlink() or not path.is_file() or path == app / "Contents/MacOS/aDrive":
            continue
        with path.open("rb") as stream:
            is_macho = stream.read(4) in magic
        if is_macho:
            sign(path)
            signed.append(str(path.relative_to(app)))
    bundles = [path for path in app.rglob("*") if not path.is_symlink() and path.is_dir() and path.suffix in (".app", ".framework", ".xpc")]
    for path in sorted(bundles, key=lambda path: len(path.parts), reverse=True):
        sign(path)
        signed.append(str(path.relative_to(app)))
    print(json.dumps({"signed": signed, "preservedFlags": True, "localLibraryEntitlement": args.local_libraries}))
