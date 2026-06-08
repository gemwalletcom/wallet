#!/usr/bin/env python3
"""Dump a Gem Keystore v4 (`.gemk`) file's plaintext header to JSON for debugging.

The v4 header is plaintext and inspectable without a password; the secret stays in the
encrypted body, which this tool only reports as a length (it never decrypts anything).

File layout (see core/crates/gem_keystore/README.md):
    offset 0   4 bytes   magic "GEMK"
    offset 4   1 byte    format version (4)
    offset 5   4 bytes   header length, big-endian u32
    offset 9   N bytes   Borsh-encoded header
    offset 9+N rest      AES-256-GCM ciphertext + 16-byte tag

Usage:
    python3 dump_v4_keystore.py <file.gemk> [-o out.json]
    python3 dump_v4_keystore.py <dir>/v4/        # dumps every .gemk in a directory
"""

import argparse
import datetime
import json
import struct
import sys
from pathlib import Path

MAGIC = b"GEMK"
VERSION_V4 = 4
PREFIX_LEN = 9
ARGON2_SALT_LEN = 16
AES_GCM_NONCE_LEN = 12

SECRET_KIND = {0: "mnemonic", 1: "private_key"}


class BorshReader:
    """Minimal Borsh decoder for the fields used by the v4 header (little-endian)."""

    def __init__(self, data: bytes):
        self.data = data
        self.pos = 0

    def take(self, n: int) -> bytes:
        if self.pos + n > len(self.data):
            raise ValueError(f"header truncated: wanted {n} bytes at offset {self.pos}")
        chunk = self.data[self.pos : self.pos + n]
        self.pos += n
        return chunk

    def u8(self) -> int:
        return self.take(1)[0]

    def u32(self) -> int:
        return struct.unpack("<I", self.take(4))[0]

    def i64(self) -> int:
        return struct.unpack("<q", self.take(8))[0]

    def string(self) -> str:
        return self.take(self.u32()).decode("utf-8")


def parse_header(reader: BorshReader) -> dict:
    keystore_id = reader.string()
    kind = reader.u8()
    created_at = reader.i64()

    kdf_variant = reader.u8()
    if kdf_variant != 0:
        raise ValueError(f"unknown kdf variant {kdf_variant} (expected 0 = Argon2id)")
    kdf = {
        "algorithm": "argon2id",
        "memory_kib": reader.u32(),
        "iterations": reader.u32(),
        "parallelism": reader.u32(),
        "salt": reader.take(ARGON2_SALT_LEN).hex(),
        "output_len": reader.u32(),
    }

    cipher_variant = reader.u8()
    if cipher_variant != 0:
        raise ValueError(f"unknown cipher variant {cipher_variant} (expected 0 = Aes256Gcm)")
    cipher = {
        "algorithm": "aes-256-gcm",
        "nonce": reader.take(AES_GCM_NONCE_LEN).hex(),
        "tag_len": reader.u8(),
    }

    return {
        "keystore_id": keystore_id,
        "kind": SECRET_KIND.get(kind, f"unknown({kind})"),
        "created_at": created_at,
        "created_at_iso": datetime.datetime.fromtimestamp(created_at, datetime.timezone.utc).isoformat(),
        "kdf": kdf,
        "cipher": cipher,
    }


def dump_file(path: Path) -> dict:
    data = path.read_bytes()
    if len(data) < PREFIX_LEN:
        raise ValueError("file too short to be a v4 keystore")
    if data[0:4] != MAGIC:
        raise ValueError(f"bad magic {data[0:4]!r} (expected {MAGIC!r})")
    version = data[4]
    if version != VERSION_V4:
        raise ValueError(f"unsupported version {version} (expected {VERSION_V4})")

    header_len = struct.unpack(">I", data[5:9])[0]  # prefix length is big-endian
    header_end = PREFIX_LEN + header_len
    if header_end > len(data):
        raise ValueError("header length exceeds file size")

    header = parse_header(BorshReader(data[PREFIX_LEN:header_end]))
    body_len = len(data) - header_end
    tag_len = header["cipher"]["tag_len"]
    filename_id = path.stem
    return {
        "file": str(path),
        "file_size": len(data),
        "magic": data[0:4].decode("ascii"),
        "version": version,
        "header_len": header_len,
        "header": header,
        "body": {
            "total_len": body_len,
            "ciphertext_len": max(body_len - tag_len, 0),
            "tag_len": tag_len,
        },
        # The on-disk filename must match the authenticated header id; flag drift for debugging.
        "filename_matches_header_id": filename_id == header["keystore_id"],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Dump a Gem Keystore v4 (.gemk) header to JSON.")
    parser.add_argument("path", type=Path, help="a .gemk file or a directory containing them")
    parser.add_argument("-o", "--out", type=Path, help="write JSON here instead of stdout")
    args = parser.parse_args()

    targets = sorted(args.path.glob("*.gemk")) if args.path.is_dir() else [args.path]
    if not targets:
        print(f"no .gemk files found at {args.path}", file=sys.stderr)
        return 1

    results = []
    for target in targets:
        try:
            results.append(dump_file(target))
        except Exception as error:  # surface a per-file error instead of aborting the batch
            results.append({"file": str(target), "error": str(error)})

    output = results[0] if len(results) == 1 else results
    text = json.dumps(output, indent=2)
    if args.out:
        args.out.write_text(text + "\n")
        print(f"wrote {args.out}")
    else:
        print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
