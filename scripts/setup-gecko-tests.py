"""Download and verify the pinned official Firefox; keep its app outside the repo."""
import hashlib
import json
import os
import plistlib
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def main():
    if sys.platform != "darwin":
        raise RuntimeError("This setup currently verifies the macOS Firefox distribution only")
    entry = json.loads((ROOT / "tests/gecko/firefox.json").read_text())
    cache = Path(os.environ.get("INJECTRA_CACHE_DIR",
        Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache")) / "injectra"))
    folder = cache / "gecko-tests" / "firefox" / entry["version"]
    folder.mkdir(parents=True, exist_ok=True)
    packed = folder / ("Firefox-" + entry["version"] + ".dmg")
    if not packed.exists():
        partial = folder / (packed.name + ".part")
        print("Downloading official Firefox " + entry["version"], flush=True)
        subprocess.run(["/usr/bin/curl", "--fail", "--location", "--retry", "1",
            "--connect-timeout", "20", "--max-time", "900", "--continue-at", "-",
            "--output", str(partial), entry["url"]], check=True)
        candidate = partial
    else:
        candidate = packed
    digest = hashlib.sha512()
    with candidate.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    if digest.hexdigest() != entry["sha512"]:
        raise RuntimeError("Official release digest mismatch; download retained for diagnosis")
    if candidate != packed:
        candidate.rename(packed)
    app = folder / "distribution" / "Firefox.app"
    if not app.exists():
        app.parent.mkdir(exist_ok=True)
        mount = Path(tempfile.mkdtemp(prefix="mount-", dir=folder))
        attached = False
        try:
            subprocess.run(["/usr/bin/hdiutil", "attach", "-readonly", "-nobrowse",
                "-mountpoint", str(mount), str(packed)], check=True)
            attached = True
            subprocess.run(["/usr/bin/ditto", str(mount / "Firefox.app"), str(app)], check=True)
        finally:
            if attached:
                subprocess.run(["/usr/bin/hdiutil", "detach", str(mount)], check=True)
            if mount.exists() and not any(mount.iterdir()):
                mount.rmdir()
    info = plistlib.loads((app / "Contents/Info.plist").read_bytes())
    if info["CFBundleShortVersionString"] != entry["version"]:
        raise RuntimeError("Extracted app version differs from the pinned release")
    subprocess.run(["/usr/bin/codesign", "--verify", "--deep", "--strict", str(app)], check=True)
    signature = subprocess.run(["/usr/bin/codesign", "-d", "--verbose=4", str(app)],
        capture_output=True, text=True, check=True).stderr
    if "Authority=Developer ID Application: Mozilla Corporation" not in signature:
        raise RuntimeError("Expected Mozilla Corporation signing authority")
    assessment = subprocess.run(["/usr/sbin/spctl", "--assess", "--type", "execute", "-v", str(app)],
        capture_output=True, text=True, check=True)
    architectures = subprocess.check_output(["/usr/bin/lipo", "-archs",
        str(app / "Contents/MacOS/firefox")], text=True).strip()
    receipt = {**entry, "app": str(app), "bytes": packed.stat().st_size,
        "architectures": architectures, "signature": signature,
        "assessment": assessment.stderr.strip()}
    (folder / "verified.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"verified": True, "version": entry["version"], "app": str(app),
        "bytes": packed.stat().st_size, "architectures": architectures}), flush=True)

if __name__ == "__main__":
    main()
