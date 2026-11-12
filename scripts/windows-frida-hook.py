"""Test-only Windows hook on an explicitly owned launch; no page-support claim."""
import argparse, ctypes, json, ntpath, pathlib, sys, time
from ctypes import wintypes


def identity(pid):
    kernel=ctypes.WinDLL('kernel32',use_last_error=True)
    kernel.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];kernel.OpenProcess.restype=wintypes.HANDLE
    kernel.CloseHandle.argtypes=[wintypes.HANDLE]
    kernel.GetProcessTimes.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
    kernel.QueryFullProcessImageNameW.argtypes=[wintypes.HANDLE,wintypes.DWORD,wintypes.LPWSTR,ctypes.POINTER(wintypes.DWORD)]
    handle=kernel.OpenProcess(0x1000,False,pid)
    if not handle:raise OSError(ctypes.get_last_error(),'Cannot query owned process')
    try:
        created,exited,kernel_time,user_time=[wintypes.FILETIME()for _ in range(4)]
        if not kernel.GetProcessTimes(handle,ctypes.byref(created),ctypes.byref(exited),ctypes.byref(kernel_time),ctypes.byref(user_time)):raise OSError(ctypes.get_last_error())
        size=wintypes.DWORD(32768);path=ctypes.create_unicode_buffer(size.value)
        if not kernel.QueryFullProcessImageNameW(handle,0,path,ctypes.byref(size)):raise OSError(ctypes.get_last_error())
        return {'started':((created.dwHighDateTime<<32)|created.dwLowDateTime)+504911232000000000,'exe':ntpath.normcase(ntpath.normpath(path.value))}
    finally:kernel.CloseHandle(handle)


def main():
    parser=argparse.ArgumentParser(__doc__);parser.add_argument('--owner',type=pathlib.Path,required=True);args=parser.parse_args()
    if sys.platform!='win32':raise RuntimeError('Windows owned process only')
    owner=json.loads(args.owner.read_text(encoding='utf-8-sig'));result={'passed':False,'productBackend':False,'pageInjection':False,'platform':sys.platform,'method':'Frida attach / actual DispatchMessageW'}
    session=script=None
    try:
        expected={'started':owner['started'],'exe':ntpath.normcase(ntpath.normpath(owner['exe']))}
        if identity(owner['pid'])!=expected:raise RuntimeError('Owned process identity mismatch')
        import frida
        result['frida']=frida.__version__;session=frida.get_local_device().attach(owner['pid'])
        if identity(owner['pid'])!=expected:raise RuntimeError('Process generation changed during attach')
        script=session.create_script((pathlib.Path(__file__).resolve().parents[1]/'tests/windows/dispatch-hook.js').read_text(),runtime='qjs');script.load()
        result['agent']=script.exports_sync.start();time.sleep(2);result['before_detach']=script.exports_sync.snapshot();result['detached']=script.exports_sync.stop();time.sleep(.5);result['after_detach']=script.exports_sync.snapshot()
        assert result['agent']['pid']==owner['pid'];assert result['before_detach']['calls']>0
        assert result['detached']['restored'];assert result['detached']['calls']==result['after_detach']['calls']
        result['passed']=True
    except Exception as error:result['error']=type(error).__name__+': '+str(error)
    finally:
        if script:
            try:script.exports_sync.stop()
            except Exception:pass
            try:script.unload()
            except Exception as error:result['cleanup_error']=str(error);result['passed']=False
        if session:
            try:session.detach()
            except Exception as error:result['cleanup_error']=str(error);result['passed']=False
        print(json.dumps(result));return 0 if result['passed'] else 1
if __name__=='__main__':sys.exit(main())
