# Reproducible Android builds

Anyone can rebuild a release from its tag and check it against the published file. Nothing in this folder reads a signing key or any other secret: every script produces unsigned files, and signing and publishing happen outside this repository.

## Build

Needs Docker and git.

```bash
git checkout <tag>
android/reproducible/docker.sh fdroid
```

`docker.sh <channel>...` builds the commit that is checked out, not the working tree. It builds the toolchain image from that commit's `Dockerfile`, then sends the commit's committed files into a container of it, which builds on its own disk, so local edits, earlier build output and the host's own toolchain never reach the result. The channels of one call build in one container and share one Rust build. Docker keeps the image, so the toolchain is installed again only when a pin changes.

The files land where Gradle puts them, under `android/app/build/outputs/`:

| Channel | File |
| --- | --- |
| `fdroid`, `universal`, `solana`, `emerald` | `apk/<channel>/release/app-<channel>-release-unsigned.apk` |
| `google` | `bundle/googleRelease/app-google-release.aab` |

Every channel builds as x86-64 Linux (`linux/amd64`), which is what F-Droid's own servers are, so every release file carries the same `libgemstone.so`. The same sources compile to a different one on ARM64 Linux, so the machine type is part of the recipe and `docker.sh` sets it. On an ARM64 computer Docker runs the container under emulation, more slowly and with the same bytes as a real x86-64 machine.

Besides the commit, the build takes the channel names, the NDK version read from that commit's `android/gradle/libs.versions.toml`, and `UPDATE_URL` when it is set, for a channel that takes its update link from the environment. Nothing else from the host reaches it.

## Verify

```bash
android/reproducible/compare.py <published.apk> android/app/build/outputs/apk/fdroid/release/app-fdroid-release-unsigned.apk
```

`compare.py` compares everything outside the APK signing block. A v2 signature covers exactly those bytes, so two APKs that match there are the same build, one of them signed. The signer may pad the file with zeros so the block starts on a 4096-byte boundary; `compare.py` accepts only zeros there and reports how many.

F-Droid runs this check on every release: its servers rebuild the `fdroid` channel and publish the signed APK only when it matches.

## Layout

| File | Runs | Does |
| --- | --- | --- |
| `docker.sh <channel>...` | anywhere with Docker | Builds the image, then runs it on the checked-out commit |
| `Dockerfile` | | The toolchain image: Ubuntu by digest, the JDK, `rustup` and Android command-line tools by checksum, the NDK, the SDK build tools and platform, the Gradle distribution, and `init.sh` |
| `entrypoint.sh` | in the container | Unpacks the commit, builds each channel and hands the files back |
| `init.sh` | on Linux | Installs the Rust toolchain the root `rust-toolchain.toml` names, and `cargo-ndk` |
| `build.sh <channel>` | on Linux | One channel's Gradle build, unsigned, with the path remap that keeps build paths out of `libgemstone.so` |
| `versions.sh` | sourced | The `cargo-ndk` version |
| `fdroid/init.sh`, `fdroid/build.sh` | on F-Droid's build server | The entry points F-Droid's recipe names; they call `init.sh` and `build.sh fdroid` |
| `compare.py <apk> <apk>` | anywhere with Python 3 | The check above |
