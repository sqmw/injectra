"""Controller-side private file I/O; parsing does not inspect any process."""
import json
import os
import stat
from pathlib import Path
from errors import BridgeError
from session_model import parse_descriptor

def private_json(path):
    path = Path(path)
    info = path.lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
        raise BridgeError("invalid_input", "Session must be a private, user-owned regular file (0600)")
    if info.st_size > 1024 * 1024:
        raise BridgeError("invalid_input", "Session descriptor exceeds size limit")
    return json.loads(path.read_text(encoding="utf-8"))

def read_session(path):
    return parse_descriptor(private_json(path))

def write_session(path, descriptor):
    path = Path(path)
    data = descriptor.to_json()
    path.parent.mkdir(parents=True, exist_ok=True)
    fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as stream:
        json.dump(data, stream, ensure_ascii=False, indent=2)
