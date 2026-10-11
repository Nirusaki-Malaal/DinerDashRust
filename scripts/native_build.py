#!/usr/bin/env python3
"""Build the native game and package Android with official SDK command-line tools."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / 'dist'

def run(args, **kwargs):
    print('+', ' '.join(map(str, args)), flush=True)
    subprocess.run(list(map(str, args)), check=True, cwd=ROOT, **kwargs)

def environment():
    env = os.environ.copy()
    local = ROOT / '.toolchains/cargo/bin'
    if (ROOT / '.toolchains/rustup/toolchains/1.94.0-x86_64-unknown-linux-gnu/bin/rustc').exists():
        env['PATH'] = str(local) + os.pathsep + env['PATH']
        env['RUSTUP_HOME'] = str(ROOT / '.toolchains/rustup')
        # Reuse the regular dependency cache; the toolchain itself stays project-local.
        env['CARGO_HOME'] = os.environ.get('CARGO_HOME', str(Path.home()/'.cargo'))
        env['RUSTUP_TOOLCHAIN'] = '1.94.0'
    env.setdefault('CARGO_BUILD_JOBS', '2')
    env.setdefault('CARGO_PROFILE_RELEASE_STRIP', 'debuginfo')
    if 'CARGO_TARGET_DIR' not in env:
        if ' ' in str(ROOT):
            env['CARGO_TARGET_DIR'] = '/tmp/diner_dash_target'
        else:
            env['CARGO_TARGET_DIR'] = str(ROOT / 'target/native')
    return env

def cargo(env, target, library=False):
    target_dir = Path(env['CARGO_TARGET_DIR'])
    run(['cargo','build','--locked','--release','-p','diner-dash-native',
         '--lib' if library else '--bin', *([] if library else ['diner-dash']), '--target',target],env=env)
    return target_dir / target / 'release'

def desktop(platform, env):
    if platform == 'windows':
        if sys.platform == 'win32':
            target = env.get('CARGO_BUILD_TARGET', 'x86_64-pc-windows-msvc')
        else:
            target = 'x86_64-pc-windows-gnu'
            env['CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER'] = 'x86_64-w64-mingw32-gcc'
            env['CC_x86_64_pc_windows_gnu'] = 'x86_64-w64-mingw32-gcc'
            env['AR_x86_64_pc_windows_gnu'] = 'x86_64-w64-mingw32-ar'
    elif platform == 'macos':
        target = env.get('CARGO_BUILD_TARGET', 'aarch64-apple-darwin')
    else:
        target = 'x86_64-unknown-linux-gnu'
    out = DIST / platform
    out.mkdir(parents=True,exist_ok=True)
    build = cargo(env,target)
    name = 'diner-dash.exe' if platform == 'windows' else 'diner-dash'
    dest = out / name
    if dest.exists():
        dest.unlink()
    shutil.copy2(build/name, dest)
    if platform == 'linux':
        shutil.copy2(ROOT/'packaging/diner-dash.desktop',out)
        shutil.copy2(ROOT/'packaging/diner-dash.png',out)
    metadata(out,platform)

def metadata(out,platform):
    (out/'metadata.json').write_text(json.dumps({'name':'Diner Dash','version':'0.1.0',
        'author':'Nirusaki','description':'Made by Nirusaki','platform':platform},indent=2)+'\n')

def strip_method_parameters(b):
    import struct
    pos = 10
    cp_count, = struct.unpack('>H', b[8:10])
    cp = [None] * cp_count
    i = 1
    while i < cp_count:
        tag = b[pos]
        pos += 1
        if tag == 1:
            l, = struct.unpack('>H', b[pos:pos+2])
            pos += 2
            cp[i] = b[pos:pos+l].decode('utf-8', 'replace')
            pos += l
        elif tag in (3, 4): pos += 4
        elif tag in (5, 6): pos += 8; i += 1
        elif tag in (7, 8, 16, 19, 20): pos += 2
        elif tag in (9, 10, 11, 12, 17, 18): pos += 4
        elif tag == 15: pos += 3
        i += 1
    access_flags, this_class, super_class, interfaces_count = struct.unpack('>HHHH', b[pos:pos+8])
    pos += 8 + 2 * interfaces_count
    fields_count, = struct.unpack('>H', b[pos:pos+2])
    pos += 2
    for _ in range(fields_count):
        pos += 6
        ac, = struct.unpack('>H', b[pos:pos+2])
        pos += 2
        for _ in range(ac):
            ni, l = struct.unpack('>HI', b[pos:pos+6])
            pos += 6 + l
    methods_start = pos
    methods_count, = struct.unpack('>H', b[pos:pos+2])
    pos += 2
    out = bytearray(b[:methods_start])
    out += struct.pack('>H', methods_count)
    for _ in range(methods_count):
        m_acc, m_name, m_desc, m_attr_count = struct.unpack('>HHHH', b[pos:pos+8])
        pos += 8
        kept_attrs = []
        for _ in range(m_attr_count):
            attr_name_idx, attr_len = struct.unpack('>HI', b[pos:pos+6])
            attr_data = b[pos:pos+6+attr_len]
            pos += 6 + attr_len
            if cp[attr_name_idx] != 'MethodParameters':
                kept_attrs.append(attr_data)
        out += struct.pack('>HHH', m_acc, m_name, m_desc)
        out += struct.pack('>H', len(kept_attrs))
        for a in kept_attrs:
            out += a
    class_attr_count, = struct.unpack('>H', b[pos:pos+2])
    pos += 2
    kept_class_attrs = []
    for _ in range(class_attr_count):
        attr_name_idx, attr_len = struct.unpack('>HI', b[pos:pos+6])
        attr_data = b[pos:pos+6+attr_len]
        pos += 6 + attr_len
        if cp[attr_name_idx] != 'MethodParameters':
            kept_class_attrs.append(attr_data)
    out += struct.pack('>H', len(kept_class_attrs))
    for a in kept_class_attrs:
        out += a
    return bytes(out)

def android(env):
    sdk = Path(env.get('ANDROID_HOME',env.get('ANDROID_SDK_ROOT',str(Path.home()/'Android/Sdk'))))
    ndk_home = env.get('ANDROID_NDK_HOME')
    if ndk_home and Path(ndk_home).exists():
        ndk = Path(ndk_home)
    elif (sdk/'ndk/28.2.13676358').exists():
        ndk = sdk/'ndk/28.2.13676358'
    elif (ROOT/'.toolchains/android-ndk-r28c-minimal').exists():
        ndk = ROOT/'.toolchains/android-ndk-r28c-minimal'
    elif (sdk/'ndk').exists() and any((sdk/'ndk').glob('*')):
        ndk = sorted((sdk/'ndk').glob('*'))[-1]
    else:
        ndk = sdk/'ndk/28.2.13676358'
    
    bt_name = env.get('ANDROID_BUILD_TOOLS')
    if bt_name and (sdk/'build-tools'/bt_name).exists():
        build_tools = sdk/'build-tools'/bt_name
    elif (sdk/'build-tools/34.0.0').exists():
        build_tools = sdk/'build-tools/34.0.0'
    elif (sdk/'build-tools').exists() and any((sdk/'build-tools').glob('*')):
        build_tools = sorted((sdk/'build-tools').glob('*'))[-1]
    else:
        build_tools = sdk/'build-tools/34.0.0'

    sdk_version = env.get('ANDROID_COMPILE_SDK')
    if sdk_version and (sdk/'platforms'/f'android-{sdk_version}'/'android.jar').exists():
        android_jar = sdk/'platforms'/f'android-{sdk_version}'/'android.jar'
    elif (sdk/'platforms/android-34/android.jar').exists():
        android_jar = sdk/'platforms/android-34/android.jar'
    elif (sdk/'platforms').exists() and any((sdk/'platforms').glob('android-*/android.jar')):
        android_jar = sorted((sdk/'platforms').glob('android-*/android.jar'))[-1]
    else:
        android_jar = sdk/'platforms/android-34/android.jar'

    toolchain = ndk/'toolchains/llvm/prebuilt/linux-x86_64/bin'
    for required in [android_jar,toolchain/'clang',build_tools/'aapt',build_tools/'d8',build_tools/'apksigner']:
        if not required.exists(): raise RuntimeError(f'Missing {required}. See README.md prerequisites.')
    staging = ROOT/'.build/android'
    if staging.exists(): shutil.rmtree(staging)
    for d in ['classes','dex','java/com/nirusaki/dinerdash','java/quad_native']:
        (staging/d).mkdir(parents=True)
    # Use exactly the Java glue shipped with the locked miniquad dependency.
    result = subprocess.check_output(['cargo','metadata','--locked','--format-version','1','--filter-platform','aarch64-linux-android'],cwd=ROOT,env=env)
    packages = json.loads(result)['packages']
    quad = next(p for p in packages if p['name']=='miniquad')
    java = Path(quad['manifest_path']).parent/'java'
    main = (java/'MainActivity.java').read_text().replace('TARGET_PACKAGE_NAME','com.nirusaki.dinerdash').replace('LIBRARY_NAME','diner_dash')
    main = main.replace('QuadNative.activityOnCreate(this);','QuadNative.setSaveDirectory(getFilesDir().getAbsolutePath());\n        QuadNative.activityOnCreate(this);')
    main = main.replace('QuadNative.activityOnPause();','QuadNative.requestPause();\n        QuadNative.activityOnPause();')
    main = main.replace('super.onBackPressed();','QuadNative.surfaceOnKeyDown(KeyEvent.KEYCODE_ESCAPE);\n        QuadNative.surfaceOnKeyUp(KeyEvent.KEYCODE_ESCAPE);')
    (staging/'java/com/nirusaki/dinerdash/MainActivity.java').write_text(main)
    quad_java = (java/'QuadNative.java').read_text().replace('public class QuadNative {',
        'public class QuadNative {\n    public native static void setSaveDirectory(String path);\n    public native static void requestPause();')
    (staging/'java/quad_native/QuadNative.java').write_text(quad_java)
    targets = {'arm64-v8a':('aarch64-linux-android','aarch64-linux-android'),
               'x86_64':('x86_64-linux-android','x86_64-linux-android')}
    for abi in env.get('ANDROID_ABIS','arm64-v8a x86_64').split():
        target,compiler = targets[abi]
        build_env = env.copy()
        clang_bin = toolchain/(compiler+'26-clang')
        if not clang_bin.exists():
            clang_bin = toolchain/'clang'
        build_env['CARGO_TARGET_'+target.upper().replace('-','_')+'_LINKER'] = str(clang_bin)
        build_env['CC_'+target.replace('-','_')] = str(clang_bin)
        ar_bin = toolchain/'llvm-ar'
        if not ar_bin.exists():
            ar_bin = Path(shutil.which('llvm-ar') or 'llvm-ar')
        build_env['AR_'+target.replace('-','_')] = str(ar_bin)
        build_env['CARGO_TARGET_'+target.upper().replace('-','_')+'_RUSTFLAGS'] = '-C link-arg=-Wl,-z,max-page-size=16384'
        build = cargo(build_env,target,library=True)
        libdir = staging/'lib'/abi
        libdir.mkdir(parents=True)
        shutil.copy2(build/'libdiner_dash.so',libdir)
        strip_bin = toolchain/'llvm-strip'
        if strip_bin.exists():
            run([strip_bin,'--strip-debug',libdir/'libdiner_dash.so'])
        elif shutil.which('llvm-strip'):
            run(['llvm-strip','--strip-debug',libdir/'libdiner_dash.so'])
    run(['javac','-source','8','-target','8','-bootclasspath',android_jar,'-d',staging/'classes',
        *sorted((staging/'java').rglob('*.java'))])
    for cls in (staging/'classes').rglob('*.class'):
        cls.write_bytes(strip_method_parameters(cls.read_bytes()))
    run([build_tools/'d8','--min-api','26','--lib',android_jar,'--output',staging/'dex',
         *sorted((staging/'classes').rglob('*.class'))])
    unsigned = staging/'unsigned.apk'
    run([build_tools/'aapt','package','-f','-M',ROOT/'packaging/android/AndroidManifest.xml',
        '-S',ROOT/'packaging/android/res','-I',android_jar,'-F',unsigned])
    with zipfile.ZipFile(unsigned,'a',compression=zipfile.ZIP_DEFLATED) as apk:
        apk.write(staging/'dex/classes.dex','classes.dex')
        for lib in (staging/'lib').rglob('*.so'): apk.write(lib,lib.relative_to(staging).as_posix())
        apk.writestr('assets/metadata.json',json.dumps({'author':'Nirusaki','description':'Made by Nirusaki'}))
    aligned = staging/'aligned.apk'
    run([build_tools/'zipalign','-f','-p','4',unsigned,aligned])
    keystore = Path(env.get('APK_KEYSTORE',str(ROOT/'.build/keys/diner-dash-local.jks'))).resolve()
    custom_key = 'APK_KEYSTORE' in env
    alias = env.get('APK_KEY_ALIAS','diner-dash')
    if custom_key:
        if not keystore.exists(): raise RuntimeError(f'Signing key not found: {keystore}')
        if 'APK_STORE_PASSWORD' not in env: raise RuntimeError('Set APK_STORE_PASSWORD for your signing key.')
    else:
        env['APK_STORE_PASSWORD']='android'
        if not keystore.exists():
            keystore.parent.mkdir(parents=True,exist_ok=True)
            run(['keytool','-genkeypair','-keystore',keystore,'-storepass:env','APK_STORE_PASSWORD',
                '-keypass:env','APK_STORE_PASSWORD','-alias',alias,'-keyalg','RSA','-keysize','2048',
                '-validity','10000','-dname','CN=Nirusaki, OU=Made by Nirusaki, O=Nirusaki','-noprompt'],env=env)
            keystore.chmod(0o600)
    out = DIST/'android'
    out.mkdir(parents=True,exist_ok=True)
    signing = [build_tools/'apksigner','sign','--ks',keystore,'--ks-key-alias',alias,
        '--ks-pass','env:APK_STORE_PASSWORD']
    if 'APK_KEY_PASSWORD' in env: signing += ['--key-pass','env:APK_KEY_PASSWORD']
    run([*signing,'--out',out/'diner-dash.apk',aligned],env=env)
    run([build_tools/'apksigner','verify','--verbose',out/'diner-dash.apk'])
    run([build_tools/'zipalign','-c','4',out/'diner-dash.apk'])
    metadata(out,'android')

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('platform',choices=['linux','windows','macos','android','all'])
    args=parser.parse_args()
    run([sys.executable, ROOT / "scripts/prepare_layouts.py"])
    env=environment()
    for platform in (['linux','windows','macos','android'] if args.platform=='all' else [args.platform]):
        if platform=='android': android(env)
        else: desktop(platform,env)
    files=sorted(p for p in DIST.rglob('*') if p.is_file() and (p.suffix in ('.exe','.apk') or p.name=='diner-dash'))
    (DIST/'SHA256SUMS').write_text(''.join(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.relative_to(DIST).as_posix()}\n' for p in files))

if __name__=='__main__':
    try: main()
    except (subprocess.CalledProcessError,RuntimeError,FileNotFoundError) as error:
        print(f'Build failed: {error}',file=sys.stderr)
        sys.exit(1)
