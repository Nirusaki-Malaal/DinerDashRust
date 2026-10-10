#!/usr/bin/env python3
"""Install a small Android link toolchain from official NDK r28c, using host LLVM.

Fetch only headers, API 26 stubs, unwind, and compiler builtins via HTTP ranges.
Every extracted file is validated by ZIP CRC. No compiler binaries are downloaded.
"""
from pathlib import Path
import concurrent.futures
import os
import shlex
import shutil
import struct
import subprocess
import zipfile

ROOT=Path(__file__).resolve().parents[1]
CACHE=ROOT/'.build/ndk-linker'
DEST=ROOT/'.toolchains/android-ndk-r28c-minimal'
URL='https://dl.google.com/android/repository/android-ndk-r28c-linux.zip'
SIZE=722261334

def download(start,end,path):
    if path.exists() and path.stat().st_size==end-start:return
    subprocess.run(['curl','-fsSL','--retry','5','--retry-all-errors','--connect-timeout','15',
        '--max-time','180','-r',f'{start}-{end-1}',URL,'-o',str(path)],check=True)
    if path.stat().st_size!=end-start:raise RuntimeError('Server did not honor the requested byte range')

def main():
    compiler=shutil.which('clang')
    for tool in ['clang','llvm-ar','llvm-strip','ld.lld','curl']:
        if not shutil.which(tool):raise RuntimeError(f'{tool} is required on PATH')
    CACHE.mkdir(parents=True,exist_ok=True)
    tail=CACHE/'directory.zip'
    download(SIZE-65536,SIZE,tail)
    data=tail.read_bytes();index=data.rfind(b'PK\x05\x06')
    _,_,_,_,_,directory_size,offset,_=struct.unpack_from('<4s4H2LH',data,index)
    download(offset,SIZE,CACHE/'central.zip')
    sparse=CACHE/'ndk.zip'
    with sparse.open('wb') as f:
        f.truncate(SIZE);f.seek(offset);f.write((CACHE/'central.zip').read_bytes())
    selected=[]
    with zipfile.ZipFile(sparse) as z:
        for e in z.infolist():
            n=e.filename;b=n.rsplit('/',1)[-1]
            libs='/sysroot/usr/lib/' in n and any('/'+t+'/' in n for t in ['aarch64-linux-android','x86_64-linux-android']) and ('/26/' in n or b in ['libunwind.a','libdl.a','libm.a','libcompiler_rt-extras.a'])
            builtins='/lib/clang/19/lib/linux/' in n and (b in ['libclang_rt.builtins-aarch64-android.a','libclang_rt.builtins-x86_64-android.a','libunwind.a'])
            if not n.endswith('/') and (libs or builtins or '/sysroot/usr/include/' in n):selected.append(e)
    selected.sort(key=lambda e:e.header_offset)
    ranges=[]
    for e in selected:
        a=e.header_offset;b=a+30+len(e.filename.encode())+len(e.extra)+e.compress_size+256
        if ranges and a-ranges[-1][1]<16384 and b-ranges[-1][0]<1024*1024:ranges[-1][1]=max(b,ranges[-1][1])
        else:ranges.append([a,b])
    print(f'Downloading {sum(b-a for a,b in ranges)/1e6:.1f} MB of official NDK link files.',flush=True)
    fd=os.open(sparse,os.O_RDWR)
    def fetch(pair):
        a,b=pair;p=CACHE/f'{a}.part';download(a,b,p);os.pwrite(fd,p.read_bytes(),a)
    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:list(pool.map(fetch,ranges))
    finally:os.close(fd)
    with zipfile.ZipFile(sparse) as z:
        for e in selected:
            relative=Path(e.filename).relative_to('android-ndk-r28c')
            if '..' in relative.parts:raise RuntimeError('Unsafe archive path')
            p=DEST/relative;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(z.read(e.filename))
    base=DEST/'toolchains/llvm/prebuilt/linux-x86_64'
    (base/'bin').mkdir(parents=True,exist_ok=True)
    resource=Path(subprocess.check_output([compiler,'-print-resource-dir'],text=True).strip())
    headers=base/'lib/clang/19/include'
    if not headers.exists():headers.symlink_to(resource/'include',target_is_directory=True)
    for tool in ['clang','llvm-ar','llvm-strip']:
        p=base/'bin'/tool
        if not p.exists():p.symlink_to(shutil.which(tool))
    for triple in ['aarch64-linux-android','x86_64-linux-android']:
        p=base/'bin'/f'{triple}26-clang'
        args=[compiler,f'--target={triple}26',f'--sysroot={base/"sysroot"}',f'-resource-dir={base/"lib/clang/19"}','-fuse-ld=lld']
        p.write_text('#!/bin/sh\nexec '+shlex.join(args)+' "$@"\n');p.chmod(0o755)
    (DEST/'README.txt').write_text('Official NDK r28c headers, API 26 link stubs, unwind and builtins; compiler is host LLVM. Source: '+URL+'\n')
    print(f'Ready: {DEST}')

if __name__=='__main__':main()
