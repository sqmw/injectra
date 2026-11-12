"""Exact Windows image and process-generation check for optional probes."""
import ctypes, ntpath
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

