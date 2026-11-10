"""Fetch pinned official macOS arm64 host fixtures outside the source tree."""
import hashlib,json,os,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[1]
cache=Path(os.environ.get("INJECTRA_CACHE_DIR",Path(os.environ.get("XDG_CACHE_HOME",Path.home()/".cache"))/"injectra"))/"host-tests"
for name,entry in json.loads((root/"tests/hosts/downloads.json").read_text()).items():
    directory=cache/name/entry["version"];directory.mkdir(parents=True,exist_ok=True)
    packed=directory/entry["filename"];partial=packed.with_suffix(".part")
    if not packed.exists():
        print("Downloading pinned "+name+" "+entry["version"],flush=True)
        subprocess.run(["curl","--fail","--location","--retry","1","--max-time","240",
            "--continue-at","-","--output",str(partial),entry["url"]],check=True)
        packed=partial
    algorithm="sha256" if "sha256" in entry else "sha1"
    digest=hashlib.new(algorithm)
    with packed.open("rb") as stream:
        for chunk in iter(lambda:stream.read(1024*1024),b""):digest.update(chunk)
    if digest.hexdigest()!=entry[algorithm]:raise RuntimeError(name+" release digest mismatch; retained for diagnosis")
    final=directory/entry["filename"]
    if packed!=final:packed.rename(final)
    extracted=directory/"distribution"
    if not (directory/"ready.json").exists():
        extracted.mkdir(exist_ok=True)
        if final.suffix==".zip":
            subprocess.run(["/usr/bin/ditto","-x","-k",str(final),str(extracted)],check=True)
        else:
            subprocess.run(["/usr/bin/tar","-xjf",str(final),"-C",str(extracted)],check=True)
        (directory/"ready.json").write_text(json.dumps({"verified":entry[algorithm],"algorithm":algorithm}))
    print(name+" ready: "+str(extracted),flush=True)
