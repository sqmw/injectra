"""Archive-only path wiring; importing does not connect to or launch any target."""
from pathlib import Path
import os,sys
ROOT=Path(__file__).resolve().parent
REFERENCES=ROOT.parents[2]
SHARED=REFERENCES/'shared/electron-v8'
base=Path(os.environ.get('INJECTRA_CACHE_DIR',Path(os.environ.get('XDG_CACHE_HOME',Path.home()/'.cache'))/'injectra'))
COMMON_CACHE=Path(os.environ.get('INJECTRA_REFERENCE_CACHE',base/'references'))
CACHE=Path(os.environ.get('INJECTRA_GADGET_CACHE',COMMON_CACHE/'runs/douyin-gadget-v8'))
GENERATOR=COMMON_CACHE/'cargo-target/debug/injectra-reference-program'
GADGET_LIBRARY=COMMON_CACHE/'gadget/FridaGadget.dylib'
sys.path.insert(0,str(SHARED))
