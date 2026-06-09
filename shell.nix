# Reproducible Android build environment.
#
# Canonical verification is Linux x86_64:
#   nix-shell shell.nix --run "android/reproducible/verify.sh --source ."
let
  nixpkgs = fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/c7f47036d3df2add644c46d712d14262b7d86c0c.tar.gz";
    sha256 = "sha256-gyqXNMgk3sh+ogY5svd2eNLJ6oEwzbAeaoBrrxD0lKk=";
  };

  rustOverlay = fetchTarball {
    url = "https://github.com/oxalica/rust-overlay/archive/02061303f7c4c964f7b4584dabd9e985b4cd442b.tar.gz";
    sha256 = "sha256-k9G98qzn+7npROUaks8VqCFm7cFtEG8ulQLBBo5lItg=";
  };

  androidNixpkgs = fetchTarball {
    url = "https://github.com/tadfisher/android-nixpkgs/archive/77f1c65db15624af98a6b670e386f4cb57b2d62b.tar.gz";
    sha256 = "sha256-Sa+pki/B361TigSv2Ipkj0qSoJw8P44c4rUjqhmiUQs=";
  };

  pkgs = import nixpkgs {
    overlays = [ (import rustOverlay) ];
    config = {
      allowUnfree = true;
      android_sdk.accept_license = true;
    };
  };

  jdkMajorVersion = "21";
  jdk = pkgs.temurin-bin-21;
  jdkHome =
    if pkgs.stdenv.isDarwin then
      "${jdk}/Library/Java/JavaVirtualMachines/temurin-${jdkMajorVersion}.jdk/Contents/Home"
    else
      "${jdk}";
  jdkTrustStore = "${jdkHome}/lib/security/cacerts";
  compileSdkVersion = "37";
  buildToolsVersion = "36.0.0";
  signingBuildToolsVersion = "34.0.0";
  ndkVersion = "28.1.13356709";
  minSdkVersion = "28";

  androidSdk =
    (import androidNixpkgs {
      pkgs = pkgs // {
        openjdk = jdk;
      };
    }).sdk
      (
        sdkPkgs: with sdkPkgs; [
          platforms-android-37-0
          build-tools-36-0-0
          build-tools-34-0-0
          ndk-28-1-13356709
          cmdline-tools-latest
          platform-tools
        ]
      );
  androidSdkRoot = "${androidSdk}/share/android-sdk";

  rustToolchain = pkgs.rust-bin.stable."1.91.0".default.override {
    targets = [
      "aarch64-linux-android"
      "armv7-linux-androideabi"
    ];
  };

  ndkHost =
    if pkgs.stdenv.isLinux && pkgs.stdenv.isx86_64 then
      "linux-x86_64"
    else if pkgs.stdenv.isDarwin then
      "darwin-x86_64"
    else
      throw "Unsupported Android NDK host platform: ${pkgs.stdenv.hostPlatform.system}";

  ndkToolchainDir = "${androidSdkRoot}/ndk/${ndkVersion}/toolchains/llvm/prebuilt/${ndkHost}/bin";
in
pkgs.mkShell {
  packages = [
    jdk
    androidSdk
    rustToolchain
    pkgs.cargo-ndk
    pkgs.coreutils
    pkgs.git
    pkgs.gnugrep
    pkgs.gnutar
    pkgs.gzip
    pkgs.just
    pkgs.openssl
    pkgs.pkg-config
    pkgs.protobuf
    pkgs.apksigcopier
    pkgs.cacert
    pkgs.rsync
    pkgs.unzip
    pkgs.which
    pkgs.zip
  ];

  JAVA_HOME = jdkHome;
  ANDROID_HOME = androidSdkRoot;
  ANDROID_SDK_ROOT = androidSdkRoot;
  ANDROID_NDK_HOME = "${androidSdkRoot}/ndk/${ndkVersion}";
  ANDROID_NDK_ROOT = "${androidSdkRoot}/ndk/${ndkVersion}";
  NDK_HOME = "${androidSdkRoot}/ndk/${ndkVersion}";
  NDK_TOOLCHAIN_DIR = ndkToolchainDir;
  NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
  SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
  GEM_ANDROID_BUILD_TOOLS = "${androidSdkRoot}/build-tools/${buildToolsVersion}";
  GEM_ANDROID_SIGNING_BUILD_TOOLS = "${androidSdkRoot}/build-tools/${signingBuildToolsVersion}";
  GRADLE_OPTS = "-Dorg.gradle.java.home=${jdkHome} -Dorg.gradle.java.installations.paths=${jdkHome} -Dorg.gradle.java.installations.auto-detect=false -Dorg.gradle.java.installations.auto-download=false -Dorg.gradle.project.android.aapt2FromMavenOverride=${androidSdkRoot}/build-tools/${buildToolsVersion}/aapt2";
  JAVA_TOOL_OPTIONS = "-Djavax.net.ssl.trustStore=${jdkTrustStore} -Djavax.net.ssl.trustStorePassword=changeit -Djavax.net.ssl.trustStoreType=JKS";

  AR_aarch64_linux_android = "${ndkToolchainDir}/llvm-ar";
  AR_armv7_linux_androideabi = "${ndkToolchainDir}/llvm-ar";
  CC_aarch64_linux_android = "${ndkToolchainDir}/aarch64-linux-android${minSdkVersion}-clang";
  CC_armv7_linux_androideabi = "${ndkToolchainDir}/armv7a-linux-androideabi${minSdkVersion}-clang";
  CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = "${ndkToolchainDir}/aarch64-linux-android${minSdkVersion}-clang";
  CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER = "${ndkToolchainDir}/armv7a-linux-androideabi${minSdkVersion}-clang";

  shellHook = ''
    echo "Gem Wallet Android reproducible build environment"
    echo "  Android platform:    ${compileSdkVersion}"
    echo "  Android build-tools: ${buildToolsVersion}"
    echo "  Signing build-tools: ${signingBuildToolsVersion}"
    echo "  NDK:                 ${ndkVersion}"
    echo "  Rust:                1.91.0"
  '';
}
