# Reproducible Android build env. Usage: nix-shell --run "cd android && ./gradlew assembleUniversalRelease"
let
  # Pinned nixpkgs. Update: nix-prefetch-url --unpack https://github.com/NixOS/nixpkgs/archive/<commit>.tar.gz
  nixpkgs = fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/8110df5ad7abf5d4c0f6fb0f8f978390e77f9685.tar.gz";
    sha256 = "0y28hhfxx1w06qrvwdxiwpm7rzplmsm255y49nkn40y82vn38x0g";
  };

  pkgs = import nixpkgs {
    config = {
      allowUnfree = true;
      android_sdk.accept_license = true;
    };
  };

  androidComposition = pkgs.androidenv.composeAndroidPackages {
    platformVersions = [ "36" ];
    buildToolsVersions = [ "36.0.0" ];
    ndkVersions = [ "28.1.13356709" ];
    includeNDK = true;
    includeEmulator = false;
    includeSystemImages = false;
    includeSources = false;
  };

  androidSdk = androidComposition.androidsdk;
in
pkgs.mkShell {
  buildInputs = [
    pkgs.temurin-bin-17
    pkgs.rustup
    pkgs.cargo-ndk
    pkgs.just
    androidSdk
  ];

  ANDROID_HOME = "${androidSdk}/libexec/android-sdk";
  ANDROID_SDK_ROOT = "${androidSdk}/libexec/android-sdk";
  JAVA_HOME = "${pkgs.temurin-bin-17}";

  shellHook = ''
    echo "Gem Wallet Android build environment"
    echo "  JDK:         $(java -version 2>&1 | head -1)"
    echo "  Android SDK: $ANDROID_HOME"
  '';
}
