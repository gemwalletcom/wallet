#!/usr/bin/env python3
import struct
import sys

SIGNING_BLOCK_MAGIC = b"APK Sig Block 42"
SIGNING_BLOCK_ALIGNMENT = 4096


def payload(path):
    data = open(path, "rb").read()
    end_record = data.rfind(b"PK\x05\x06")
    directory_size, directory_offset = struct.unpack("<II", data[end_record + 12 : end_record + 20])
    entries_end = directory_offset
    if data[directory_offset - 16 : directory_offset] == SIGNING_BLOCK_MAGIC:
        block_size = struct.unpack("<Q", data[directory_offset - 24 : directory_offset - 16])[0]
        entries_end = directory_offset - block_size - 8
    return {
        "entries": data[:entries_end],
        "central directory": data[directory_offset : directory_offset + directory_size],
        "end record": data[end_record : end_record + 16] + data[end_record + 20 :],
    }


def same_entries(first, second):
    shorter, longer = sorted((first, second), key=len)
    padding = longer[len(shorter) :]
    return longer.startswith(shorter) and not any(padding) and (not padding or len(longer) % SIGNING_BLOCK_ALIGNMENT == 0)


def main():
    if len(sys.argv) != 3:
        sys.exit("usage: compare.py <apk> <apk>")
    first, second = payload(sys.argv[1]), payload(sys.argv[2])
    results = {
        "entries": same_entries(first["entries"], second["entries"]),
        "central directory": first["central directory"] == second["central directory"],
        "end record": first["end record"] == second["end record"],
    }
    for part, same in results.items():
        print(f"{part:18} {len(first[part]):>10} {len(second[part]):>10} {'same' if same else 'DIFFERENT'}")
    padding = abs(len(first["entries"]) - len(second["entries"]))
    if all(results.values()):
        print(f"identical outside the signing block ({padding} bytes of zero padding before it)" if padding else "identical outside the signing block")
    else:
        print("the APKs differ")
    sys.exit(0 if all(results.values()) else 1)


if __name__ == "__main__":
    main()
