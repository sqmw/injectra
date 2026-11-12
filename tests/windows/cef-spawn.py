"""Interactive Windows-only CEF startup probe. Run via an owned scheduled task."""
import argparse
import json
import pathlib
import sys
import time

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / 'scripts'))
from windows_frida_hook_identity import identity


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument('--exe', required=True)
    parser.add_argument('--port', type=int, required=True)
    parser.add_argument('--output', type=pathlib.Path, required=True)
    args = parser.parse_args()
    if sys.platform != 'win32':
        raise RuntimeError('Windows only')
    import frida
    args.output.mkdir(parents=True, exist_ok=False)
    events = []
    device = frida.get_local_device()
    session = script = None
    child = None
    try:
        child = device.spawn([args.exe, '--remote-debugging-address=127.0.0.1',
                              '--remote-debugging-port=' + str(args.port)], cwd=str(pathlib.Path(args.exe).parent))
        owner = dict(identity(child), pid=child)
        # Exact process generation, reusable by the existing cleanup reconciler.
        (args.output / 'launch.json').write_text(json.dumps(owner))
        session = device.attach(child)
        if identity(child) != {k:owner[k] for k in ('started','exe')}:
            raise RuntimeError('Spawn generation changed')
        script = session.create_script(pathlib.Path(__file__).with_name('cef-init.js').read_text(), runtime='qjs')
        script.on('message', lambda message, data: events.append(message))
        script.load()
        events.append(script.exports_sync.start(args.port))
        device.resume(child)
        for _ in range(1200):
            if (args.output / 'stop').exists():
                break
            time.sleep(.1)
    except Exception as error:
        events.append({'error':str(error)})
        # Resume a suspended process even when instrumentation could not start.
        if child:
            try: device.resume(child)
            except Exception: pass
    finally:
        if script:
            try: events.append(script.exports_sync.stop())
            except Exception as error: events.append({'detach_error':str(error)})
        if session:
            try: session.detach()
            except Exception as error: events.append({'detach_error':str(error)})
        (args.output / 'probe.json').write_text(json.dumps({'product_backend':False,'events':events},indent=2))


if __name__ == '__main__':
    main()
