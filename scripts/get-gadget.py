"""Explicit dependency preparation; never starts or attaches to an application."""
from pathlib import Path
import hashlib,lzma,os,subprocess
base=Path(os.environ.get('INJECTRA_CACHE_DIR',Path(os.environ.get('XDG_CACHE_HOME',Path.home()/'.cache'))/'injectra'))
cache=base/'frida/gadget'
cache.mkdir(parents=True,exist_ok=True)
packed=cache/'FridaGadget.dylib.xz'
url='https://github.com/frida/frida/releases/download/17.25.2/frida-gadget-17.25.2-macos-universal.dylib.xz'
expected='e364bc62c7628d6ad34d7ce04134702f2585875bce9d7abc1552bee21657b568'
if not packed.exists():
    command=['curl','--fail','--location','--max-time','90']
    if os.environ.get('GADGET_DOWNLOAD_PROXY'):
        command+=['--proxy',os.environ['GADGET_DOWNLOAD_PROXY']]
    temporary=packed.with_suffix('.part')
    subprocess.run(command+['--output',str(temporary),url],check=True)
    if hashlib.sha256(temporary.read_bytes()).hexdigest()!=expected:
        raise RuntimeError('Gadget release digest mismatch; partial data retained')
    temporary.replace(packed)
raw=packed.read_bytes()
if hashlib.sha256(raw).hexdigest()!=expected:
    raise RuntimeError('Cached Gadget digest mismatch')
library=cache/'FridaGadget.dylib'
decoded=lzma.decompress(raw)
if library.exists() and library.read_bytes()!=decoded:
    raise RuntimeError('Existing Gadget differs from the verified release; not overwritten')
if not library.exists():
    library.write_bytes(decoded)
print('Official Gadget digest verified; no application was started')
