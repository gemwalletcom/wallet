# Android Reproducible Builds

This document is the target design for Gem Wallet Android reproducible releases.
The canonical build is Linux x86_64 under Nix. macOS can enter the same Nix
shell for development smoke tests, but a macOS result is not a reproducibility
proof because the Android NDK host toolchain and native linkers differ by OS.
Docker is not part of the release design.

## Goals

1. Build the release APK from source in a pinned Linux toolchain.
2. Rebuild the same source in a second clean checkout and get identical unsigned
   APK bytes.
3. Sign the exact unsigned APK as a final step, preferably on Linux with the same
   pinned Android build-tools.
4. Allow F-Droid to rebuild the `fdroidRelease` flavor and verify our published
   developer-signed APK with signature copying.
5. Fail the build if R8 emits a path-sensitive or map-id-derived `SourceFile`
   value into DEX.

## Canonical Toolchain

The pinned environment lives in `shell.nix`.

| Component | Version | Source of truth |
| --- | --- | --- |
| Nixpkgs | `c7f47036d3df2add644c46d712d14262b7d86c0c` | `shell.nix` |
| Android SDK package set | `tadfisher/android-nixpkgs@77f1c65db15624af98a6b670e386f4cb57b2d62b` | `shell.nix` |
| Rust overlay | `02061303f7c4c964f7b4584dabd9e985b4cd442b` | `shell.nix` |
| Rust | `1.91.0` | `shell.nix` |
| Gradle | `9.4.1` | `android/gradle/wrapper/gradle-wrapper.properties` |
| Android Gradle Plugin | `9.2.1` | `android/gradle/libs.versions.toml` |
| Kotlin | `2.3.21` | `android/gradle/libs.versions.toml` |
| JDK | Temurin 21 for Gradle; Java 17 bytecode target | `shell.nix`, `android/gradle/gradle-daemon-jvm.properties`, and Android Gradle config |
| Android platform | `37` | `android/app/build.gradle.kts` and `shell.nix` |
| Build tools for build/aapt2 | `36.0.0` | `shell.nix`, matching AGP 9.2 default |
| Build tools for APK signing | `34.0.0` | `shell.nix`, for apksigcopier-compatible signatures |
| NDK | `28.1.13356709` | `android/app/build.gradle.kts` and `shell.nix` |
| Min SDK | `28` | `android/app/build.gradle.kts` and NDK linker names |
| cargo-ndk | Nixpkgs-pinned package | `shell.nix` |

AGP 9.2 requires Gradle 9.4.1, and Android API 37 requires AGP 9.1.1 or newer.
The current branch already satisfies both constraints. See Android's AGP
compatibility tables for the Gradle and API-level requirements:
https://developer.android.com/build/releases/about-agp

## Build Outputs

The reproducible scripts build unsigned APKs first:

```bash
nix-shell shell.nix --run "android/reproducible/build-apk.sh"
nix-shell shell.nix --run "android/reproducible/build-apk.sh --variant fdroid"
```

The expected output paths are:

```text
android/app/build/outputs/apk/fdroid/release/app-fdroid-release-unsigned.apk
android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk
```

The `fdroid` flavor is the F-Droid target. It uses `pushes-stub` instead of FCM
and `review-stub` instead of Play Review. During `FDROID_BUILD=true`, the app
does not apply the Google Services Gradle plugin and the root build excludes
`mavenLocal()`. JitPack remains enabled because current Reown/WalletConnect
dependencies resolve several transitive artifacts from JitPack. The F-Droid APK
is ARM-only (`armeabi-v7a` and `arm64-v8a`) to match Gem's existing release ABI
surface.

The `universal` flavor is the direct APK distribution target. It still includes
the normal Gem direct-distribution behavior and is not the F-Droid recipe.

## Verification Flow

Run the local two-build verifier on Linux:

```bash
nix-shell shell.nix --run "android/reproducible/verify.sh --source ."
nix-shell shell.nix --run "android/reproducible/verify.sh --source . --variant fdroid"
```

On macOS, use the same Nix shell for smoke/debug builds:

```bash
nix-shell shell.nix --run "android/reproducible/build-apk.sh --output /tmp/gem-universal.apk"
nix-shell shell.nix --run "android/reproducible/build-apk.sh --variant fdroid --output /tmp/gem-fdroid.apk"
```

You can run `verify.sh --allow-non-linux` on macOS as an exploratory diagnostic,
but it is not release evidence. The macOS NDK host tools and linker are
different from Linux x86_64, and DEX/R8 differences found there must be
confirmed in Linux CI before treating them as release blockers.

The verifier does this:

1. Copies the source tree to two independent temporary directories with build
   outputs and VCS metadata removed.
2. Builds the selected release variant in both copies with `SKIP_SIGN=true`.
3. Sets `SOURCE_DATE_EPOCH` from the source commit time.
4. Adds Rust `--remap-path-prefix` flags for the checkout, Cargo home, Android
   SDK, and Android NDK paths.
5. Scans every `classes*.dex` for `r8-map-id-`.
6. Compares the unsigned APK SHA-256 values. This catches broader R8/DEX
   instability, including changes to R8 marker metadata such as `pg-map-id`.
7. Prints changed APK entries and R8 marker strings from both APKs if the
   unsigned bytes differ.
8. Optionally copies the official APK signature onto the rebuilt unsigned APK
   and compares the signed bytes:

```bash
nix-shell shell.nix --run \
  "android/reproducible/verify.sh --source . --variant fdroid --official gem_wallet_fdroid_2.80.apk"
```

If the unsigned APKs differ, the script writes a `diffoscope` HTML report when
`diffoscope` is installed in the surrounding environment.

## R8 SourceFile / Map-Id Policy

R8 can emit synthetic `SourceFile` values like `r8-map-id-...` when retracing
metadata is required. Those values couple DEX output to the generated mapping
state. Gem avoids post-processing DEX or patching APK contents after the build.
Instead, the app ProGuard rules force a stable `SourceFile` value:

```proguard
-keepattributes SourceFile
-renamesourcefileattribute SourceFile
```

Store release builds also include `android/app/proguard-stacktrace-rules.pro`,
which keeps `LineNumberTable` for crash line-number retracing. F-Droid and
`ANDROID_REPRODUCIBLE_BUILD=true` builds intentionally omit that file. Linux CI
proved R8 9.2.14 can otherwise make different line-position, `outlineCallsite`,
and `residualsignature` mapping metadata across two clean builds of the same
source. Those differences changed `classes.dex`, `classes2.dex`, and
`assets/dexopt/baseline.prof`.

Reproducible builds also include `android/app/proguard-reproducible-rules.pro`,
which applies `-dontoptimize`. AGP 9 no longer allows
`getDefaultProguardFile("proguard-android.txt")`, so the supported form is the
normal optimized default ProGuard file plus this reproducible-only rule file.
This keeps shrinking and obfuscation enabled while removing the R8 optimization
phase that produced different `pg-map-id` values between clean builds.

The verifier treats any `r8-map-id-` string in DEX as a release blocker. This is
the direct test for the historical `SourceFile` issue. R8 also writes marker
metadata such as `~~R8{...,"pg-map-id":"..."}` into DEX. That marker is allowed
to exist, but it must be stable as part of the full unsigned APK byte comparison
on Linux.

## Signing

Signing is not macOS-specific. Android APK signing is Java/Android build-tools
work and can run on Linux. The release design is:

1. Build unsigned APKs on Linux in Nix.
2. Compare unsigned outputs before signing.
3. Sign the exact unsigned artifact on Linux with pinned build-tools 34:

```bash
ANDROID_KEYSTORE_PASSWORD=... nix-shell shell.nix --run \
  "android/reproducible/sign-apk.sh \
    --unsigned app-fdroid-release-unsigned.apk \
    --out gem_wallet_fdroid_2.80.apk \
    --ks release.keystore \
    --ks-key-alias \"$ANDROID_KEYSTORE_ALIAS\""
```

Build-tools 34 is intentionally used for signing because F-Droid documents that
signatures from newer `apksigner` versions can fail `apksigcopier` verification:
https://f-droid.org/docs/Reproducible_Builds/

The release keystore should never be present during the reproducible unsigned
comparison step. Keep it in the final signing job only.

## F-Droid Setup

The F-Droid recipe template is in:

```text
android/fdroid-build/metadata/com.gemwallet.android.yml
```

The recipe builds `:app:assembleFdroidRelease`, sets `FDROID_BUILD=true`, uses
NDK `28.1.13356709`, installs Rust `1.91.0`, installs Android Rust targets, and
writes Cargo linker configuration for the F-Droid Linux build server.

For a real fdroiddata merge request, update these fields for every release:

```yaml
versionName: '2.80'
versionCode: 779
commit: '2.80'
binary: https://github.com/gemwalletcom/wallet/releases/download/%v/gem_wallet_fdroid_%v.apk
CurrentVersion: '2.80'
CurrentVersionCode: 779
```

After the first developer-signed F-Droid APK is published, extract signatures in
fdroiddata:

```bash
fdroid signatures https://github.com/gemwalletcom/wallet/releases/download/2.80/gem_wallet_fdroid_2.80.apk
```

F-Droid's reproducible build model copies signatures from the developer APK to
the rebuilt unsigned APK and verifies the result, so the unsigned APK bytes must
match exactly apart from the signature block. F-Droid's metadata fields and
`binary:` verification are documented here:
https://f-droid.org/en/docs/Build_Metadata_Reference/

## Known Non-Determinism Sources And Controls

| Source | Control |
| --- | --- |
| R8 `SourceFile` map ids | `-renamesourcefileattribute SourceFile` and DEX scan |
| R8 line-position / `pg-map-id` instability | omit `LineNumberTable` and add `-dontoptimize` for reproducible builds; two-build unsigned APK comparison on Linux; verifier prints R8 markers on mismatch |
| Absolute Rust paths | `--remap-path-prefix` in reproducible scripts |
| Host NDK differences | canonical Linux x86_64 build only |
| Extra native ABIs | reproducible/F-Droid scripts build only `arm64-v8a,armeabi-v7a` |
| Signing block bytes | sign after unsigned comparison with build-tools 34 |
| Gradle daemon/caches | `--no-daemon`, clean source copies, isolated `GRADLE_USER_HOME` |
| Google/Firebase resources in F-Droid | `FDROID_BUILD=true` disables Google Services plugin |
| Local repositories | `FDROID_BUILD=true` disables `mavenLocal()` |
| JitPack transitive dependencies | currently required by Reown/WalletConnect; replace or vendor before an F-Droid review if policy requires it |
| Prebuilt native wallet-core | no tracked wallet-core `.so`, `.aar`, or `.jar` artifacts |

## CI Design

`.github/workflows/android-reproducible.yml` runs the Linux Nix verifier. The
default workflow variant is `universal`, matching direct APK releases. Use
manual dispatch with `fdroid` when validating the F-Droid flavor.
Do not use local Docker as a substitute on Apple Silicon; that produces an
`aarch64-linux` environment, while the Android NDK Linux host used for
reproducible releases is x86_64 Linux. Android's NDK host tag table documents
Linux as `linux-x86_64`, while macOS keeps the historical `darwin-x86_64` tag
for fat binaries that include Apple Silicon support:
https://developer.android.com/ndk/guides/other_build_systems

The release repo should eventually split Android from the current macOS
self-hosted runner:

1. Linux Nix job builds `fdroidRelease` and `universalRelease` unsigned APKs.
2. Linux Nix verifier compares two clean builds.
3. Linux signing job downloads the verified unsigned artifact and signs it.
4. Upload jobs publish the signed APKs to GitHub Releases, R2, F-Droid binary
   URL, and store-specific pipelines.
5. macOS self-hosted runners remain only for iOS/Xcode release jobs.
