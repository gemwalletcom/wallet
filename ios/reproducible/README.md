# Reproducible iOS builds

Anyone with a Mac can rebuild a release from its tag and check it against the IPA attached to the GitHub release, and check that IPA against the copy the App Store delivers. Nothing in this folder reads a signing key or any other secret: `build.sh` produces an unsigned app, and signing and uploading happen outside this repository.

## Build

Needs an Apple Silicon Mac with Xcode, `rustup`, `just`, `xcbeautify` and git.

```bash
git checkout <tag>
git submodule update --init
ios/reproducible/build.sh
```

`build.sh` builds the commit that is checked out, not the working tree. It copies that commit's committed files, and every submodule at the commit the tag records, into `/private/tmp/gem-wallet-ios` and builds there with an almost empty environment, so local edits, earlier build output and the caller's variables never reach the result. The unsigned app lands in `ios/build/reproducible/Gem.app`.

Three things outside the commit decide the bytes, and the script fixes or reports each:

| Input | Why it matters | How it is fixed |
| --- | --- | --- |
| The folder the build runs in | The Swift compiler writes the length of a source file's path into the executable wherever it checks that a closure did not escape, so the same commit built under a path of another length differs in those instructions | The build always runs in `/private/tmp/gem-wallet-ios` |
| The home folder and Rust install | Rust panic messages carry the paths of dependency and standard library sources | Both are remapped, to `/cargo` and to the `/rustc/<commit>` path a toolchain without `rust-src` uses |
| The Xcode build | Another compiler or SDK gives another executable | The script prints the Xcode it uses; the `DTXcodeBuild` key in the published `Info.plist` names the one the release used. Select it with `xcode-select` or `DEVELOPER_DIR` |

The Rust toolchain is the one the root `rust-toolchain.toml` names; `rustup` installs it on first use.

## Verify a rebuild

```bash
ios/reproducible/compare.py <published.ipa> ios/build/reproducible/Gem.app
```

`compare.py` takes two apps, each an `.ipa` or an `.app` folder, and compares every file in the app bundle. It prints a line for each file that is not byte for byte the same and exits with 0 only when none of them is a difference. It refuses an IPA that holds more than one app or two entries with one name, and any app that holds a symbolic link. A file that is an executable is never accepted under one of the statuses below except the first:

| Status | Meaning |
| --- | --- |
| `same outside the signature` | An executable that matches once the code signature, the size fields that record it and the encryption flag are set aside |
| `signing file` | `_CodeSignature/CodeResources` or `embedded.mobileprovision`, which only a signed app has |
| `same contents` | An `Assets.car` whose renditions, as `assetutil` lists them, are the same. The asset catalog compiler orders the tables of a catalog with icon appearances differently on every run, so the app's own catalog never matches byte for byte. Needs Xcode; without it the file is reported as `DIFFERENT` |
| `same apart from the build machine` | An `Info.plist` that differs only in `BuildMachineOSBuild`, the macOS build of the machine |
| `DIFFERENT` | Anything else, with the sections of an executable or the keys of an `Info.plist` that differ |

## Verify the App Store copy

```bash
ios/reproducible/compare.py --app-store <published.ipa> <App Store copy>
```

The App Store copy goes second. `--app-store` accepts what the store adds or rewrites, each as `App Store file`, and nothing wider:

- Files in `SC_Info/` at the root of the app.
- An `Info.plist` at the root that differs only in `DTAppStoreToolsBuild` and `UISupportedDevices`. Any other key is a difference and is printed.
- An `Assets.car` the store thinned for one device: every rendition it holds is in the published catalog.

The store re-signs the executable and encrypts its code. A copy as downloaded still has that range encrypted: the script compares everything else in the executable, reports how many bytes it could not compare, and exits with 2. A decrypted copy is compared in full.

## What a match does not cover

- Entitlements and the provisioning profile, which live in the code signature.
- Whatever an IPA holds outside the app bundle, such as `Symbols/`.
- Bytes in an `Assets.car` that `assetutil` does not list.

## Layout

| File | Does |
| --- | --- |
| `build.sh` | Copies the checked-out commit to the build folder, builds the Rust library and archives the `Gem` scheme unsigned |
| `compare.py <app> <app>` | The checks above |
