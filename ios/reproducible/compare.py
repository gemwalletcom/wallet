#!/usr/bin/env python3
import json
import plistlib
import shutil
import struct
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

MACH_O_64 = b"\xcf\xfa\xed\xfe"
HEADER_SIZE = 32
SEGMENT_64 = 0x19
CODE_SIGNATURE = 0x1D
ENCRYPTION_INFO_64 = 0x2C
BUILD_MACHINE_KEYS = {"BuildMachineOSBuild"}
USAGE = "usage: compare.py [--app-store] <ipa or app> <ipa or app>"


def bundle_files(path):
    source = Path(path)
    if source.is_dir():
        return {str(file.relative_to(source)): file.read_bytes() for file in source.rglob("*") if file.is_file()}
    with zipfile.ZipFile(source) as archive:
        entries = ((entry, entry.filename.split("/", 2)) for entry in archive.infolist() if not entry.is_dir())
        return {parts[2]: archive.read(entry) for entry, parts in entries if len(parts) == 3 and parts[0] == "Payload" and parts[1].endswith(".app")}


def signing_file(name):
    return name.endswith("_CodeSignature/CodeResources") or name.endswith("embedded.mobileprovision")


def app_store_file(name):
    return name == "Info.plist" or Path(name).name == "Assets.car" or "SC_Info/" in name


def load_commands(data):
    count = struct.unpack_from("<I", data, 16)[0]
    offset = HEADER_SIZE
    for _ in range(count):
        command, size = struct.unpack_from("<II", data, offset)
        yield command, offset, size
        offset += size


def encrypted_size(data):
    for command, offset, _ in load_commands(data):
        if command == ENCRYPTION_INFO_64:
            size, identifier = struct.unpack_from("<II", data, offset + 12)
            return size if identifier else 0
    return 0


def segment_name(data, offset):
    return data[offset : offset + 16].rstrip(b"\0").decode()


def without_signature(data, encrypted):
    image = bytearray(data)
    commands = []
    end = len(data)
    for command, offset, size in load_commands(data):
        body = bytearray(data[offset : offset + size])
        if command == CODE_SIGNATURE:
            end = struct.unpack_from("<I", body, 8)[0]
            continue
        if command == ENCRYPTION_INFO_64:
            crypt_offset, crypt_size = struct.unpack_from("<II", body, 8)
            struct.pack_into("<I", body, 16, 0)
            if encrypted:
                image[crypt_offset : crypt_offset + crypt_size] = bytes(crypt_size)
        if command == SEGMENT_64 and segment_name(body, 8) == "__LINKEDIT":
            struct.pack_into("<Q", body, 32, 0)
            struct.pack_into("<Q", body, 48, 0)
        commands.append(bytes(body))
    commands_end = HEADER_SIZE + struct.unpack_from("<I", data, 20)[0]
    joined = b"".join(commands)
    image[16:24] = struct.pack("<II", len(commands), len(joined))
    image[HEADER_SIZE:commands_end] = joined.ljust(commands_end - HEADER_SIZE, b"\0")
    return bytes(image[:end]).rstrip(b"\0")


def parts(data, image):
    commands_end = HEADER_SIZE + struct.unpack_from("<I", image, 20)[0]
    found = {"load commands": image[:commands_end]}
    for command, offset, _ in load_commands(data):
        if command != SEGMENT_64:
            continue
        segment = segment_name(data, offset + 8)
        if segment == "__LINKEDIT":
            file_offset = struct.unpack_from("<Q", data, offset + 40)[0]
            found[segment] = image[file_offset:]
        for index in range(struct.unpack_from("<I", data, offset + 64)[0]):
            section = offset + 72 + index * 80
            size, file_offset = struct.unpack_from("<QI", data, section + 40)
            if file_offset:
                found[f"{segment},{segment_name(data, section)}"] = image[file_offset : file_offset + size]
    return found


def differing_parts(first, second, first_image, second_image):
    first_parts, second_parts = parts(first, first_image), parts(second, second_image)
    for name in dict.fromkeys([*first_parts, *second_parts]):
        one, other = first_parts.get(name, b""), second_parts.get(name, b"")
        if one != other:
            changed = sum(a != b for a, b in zip(one, other)) + abs(len(one) - len(other))
            yield f"{name}: {len(one)} and {len(other)} bytes, {changed} differ"


def differing_keys(first, second):
    one, other = plistlib.loads(first), plistlib.loads(second)
    return sorted(key for key in one.keys() | other.keys() if one.get(key) != other.get(key))


def catalog_contents(data):
    with tempfile.NamedTemporaryFile(suffix=".car") as file:
        file.write(data)
        file.flush()
        listing = json.loads(subprocess.run(["xcrun", "assetutil", "--info", file.name], capture_output=True, check=True).stdout)
    return sorted(json.dumps({key: value for key, value in entry.items() if key != "Timestamp"}, sort_keys=True) for entry in listing)


def compare(name, first, second, app_store):
    expected = "signing file" if signing_file(name) else "App Store file" if app_store and app_store_file(name) else None
    if first is None or second is None:
        return expected or "DIFFERENT", [f"only in the {'second' if first is None else 'first'}"], 0
    if first == second:
        return "same", [], 0
    if first[:4] == MACH_O_64 and second[:4] == MACH_O_64:
        encrypted = max(encrypted_size(first), encrypted_size(second))
        first_image, second_image = without_signature(first, encrypted), without_signature(second, encrypted)
        if first_image == second_image:
            return "same outside the signature", [f"{encrypted} bytes of encrypted code not compared"] if encrypted else [], encrypted
        return "DIFFERENT", list(differing_parts(first, second, first_image, second_image)), encrypted
    kind = Path(name).name
    if kind == "Assets.car" and shutil.which("xcrun") and catalog_contents(first) == catalog_contents(second):
        return "same contents", ["compared as the renditions assetutil lists"], 0
    if kind == "Info.plist":
        keys = differing_keys(first, second)
        details = [f"keys that differ: {', '.join(keys)}"]
        if not keys:
            return "same contents", [], 0
        if set(keys) <= BUILD_MACHINE_KEYS:
            return "same apart from the build machine", details, 0
        return expected or "DIFFERENT", details, 0
    return expected or "DIFFERENT", [], 0


def main():
    paths = [argument for argument in sys.argv[1:] if argument != "--app-store"]
    app_store = len(paths) < len(sys.argv) - 1
    if len(paths) != 2:
        sys.exit(USAGE)
    first, second = bundle_files(paths[0]), bundle_files(paths[1])
    if not first or not second:
        sys.exit(f"no app found in {paths[1] if first else paths[0]}")
    names = sorted(first.keys() | second.keys())
    results = {name: compare(name, first.get(name), second.get(name), app_store) for name in names}
    for name, (status, details, _) in results.items():
        if status == "same":
            continue
        print(f"{name:48} {len(first.get(name, b'')):>10} {len(second.get(name, b'')):>10} {status}")
        for detail in details:
            print(f"    {detail}")
    statuses = [status for status, _, _ in results.values()]
    encrypted = sum(size for _, _, size in results.values())
    print(f"{statuses.count('same')} of {len(names)} files are byte for byte the same")
    if "DIFFERENT" in statuses:
        print("the apps differ")
        sys.exit(1)
    if encrypted:
        print(f"no difference found, but {encrypted} bytes of encrypted code could not be compared")
        sys.exit(2)
    print("the apps are byte for byte the same" if statuses.count("same") == len(names) else "the apps match outside the parts listed above")


if __name__ == "__main__":
    main()
