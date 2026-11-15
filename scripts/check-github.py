"""Validate repository metadata without accessing user apps or GitHub credentials."""
import hashlib
import json
import re
import urllib.request
from pathlib import Path

import jsonschema
import yaml

ROOT = Path(__file__).resolve().parents[1]


def load(path, loader=yaml.BaseLoader):
    with path.open(encoding="utf-8") as source:
        return yaml.load(source, Loader=loader)


for path in sorted((ROOT / ".github").rglob("*.yml")):
    document = load(path)
    assert isinstance(document, dict), f"Expected mapping: {path}"
    if path.parent.name == "workflows":
        assert document.get("permissions", {}).get("contents") == "read", path
        assert "pull_request_target" not in document.get("on", {}), path
        for job in document["jobs"].values():
            assert "timeout-minutes" in job, path
            for step in job["steps"]:
                if "uses" in step:
                    assert re.fullmatch(r"[\w.-]+/[\w./-]+@[a-f0-9]{40}", step["uses"]), step
    elif path.parent.name == "ISSUE_TEMPLATE" and "body" in document:
        ids = [item["id"] for item in document["body"] if "id" in item]
        assert len(ids) == len(set(ids)), path
        assert document.get("name") and document.get("description"), path
        for item in document["body"]:
            assert item.get("type") in {"input", "textarea", "dropdown", "checkboxes", "markdown"}, item

source = json.loads((ROOT / ".github/cff-schema-source.json").read_text())
with urllib.request.urlopen(source["url"], timeout=30) as response:
    schema_bytes = response.read()
assert hashlib.sha256(schema_bytes).hexdigest() == source["sha256"], "CFF schema checksum mismatch"
citation = load(ROOT / "CITATION.cff", yaml.SafeLoader)
jsonschema.Draft7Validator(json.loads(schema_bytes)).validate(citation)
assert citation["type"] == "software" and "preferred-citation" not in citation
assert "doi" not in citation, "No published DOI has been confirmed"
print("GitHub YAML/forms/action pins and official CFF 1.2 schema: OK")
