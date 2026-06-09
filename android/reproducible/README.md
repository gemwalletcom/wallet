# Reproducible Builds

Gem Wallet Android reproducible builds use the root `shell.nix` and Linux
x86_64 as the canonical environment.

Run the default direct APK reproducibility check:

```bash
nix-shell shell.nix --run "android/reproducible/verify.sh --source ."
```

Run the F-Droid reproducibility check:

```bash
nix-shell shell.nix --run "android/reproducible/verify.sh --source . --variant fdroid"
```

On macOS, use Nix for smoke/debug builds:

```bash
nix-shell shell.nix --run "android/reproducible/build-apk.sh --output /tmp/gem-universal.apk"
nix-shell shell.nix --run "android/reproducible/build-apk.sh --variant fdroid --output /tmp/gem-fdroid.apk"
```

The two-build byte comparison is Linux-only release evidence. A macOS two-build
run can still be useful for debugging, but it is not expected to be the
authoritative reproducibility result.

Run the direct APK check explicitly:

```bash
nix-shell shell.nix --run "android/reproducible/verify.sh --source . --variant universal"
```

See [`docs/android-reproducible-builds.md`](../../docs/android-reproducible-builds.md)
for the full design, R8 policy, Linux signing flow, and F-Droid setup.
