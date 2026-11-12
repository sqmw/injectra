"""Bounded PowerShell diagnostics over SSH; avoids cmd length and stdin EOF traps."""
import base64
import gzip
import json
import subprocess


def run_powershell(ssh, script, timeout=60):
    script="$ProgressPreference='SilentlyContinue';$ErrorActionPreference='Stop';[Console]::OutputEncoding=[Text.Encoding]::UTF8;"+script
    packed=base64.b64encode(gzip.compress(script.encode())).decode()
    bootstrap="$b=[Convert]::FromBase64String('"+packed+"');$m=[IO.MemoryStream]::new($b,0,$b.Length);$g=[IO.Compression.GZipStream]::new($m,[IO.Compression.CompressionMode]::Decompress);$r=[IO.StreamReader]::new($g,[Text.Encoding]::UTF8);Invoke-Expression ($r.ReadToEnd())"
    command='powershell.exe -NoProfile -NonInteractive -EncodedCommand '+base64.b64encode(bootstrap.encode('utf-16le')).decode()
    if len(command)>7800:raise RuntimeError('Diagnostic command exceeds Windows command-line limit')
    result=subprocess.run(ssh+[command],capture_output=True,text=True,errors='replace',timeout=timeout)
    if result.returncode:raise RuntimeError(result.stderr[-2400:])
    return json.loads(result.stdout.lstrip('\ufeff'))


def quoted(value):
    return "'"+str(value).replace("'","''")+"'"
