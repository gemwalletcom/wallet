FROM ubuntu@sha256:a2d43bffc46670d7108bb3c0468c67c5557c4c92e83e1e34cbff1322ad1b94dc AS amd64
ADD --checksum=sha256:8b9b611d917e3dfd8419d3566b5949226f69b052fed7e109c1368add1dfe8711 https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.21%2B9/OpenJDK17U-jdk_x64_linux_hotspot_17.0.21_9.tar.gz /tmp/jdk.tar.gz
ADD --checksum=sha256:b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55 https://static.rust-lang.org/rustup/archive/1.29.3/x86_64-unknown-linux-gnu/rustup-init /tmp/rustup-init

FROM ubuntu@sha256:18b33ff3829707f31ada654a963b9ee35f2e6f0241b964bd17a22a9b330cb63d AS arm64
ADD --checksum=sha256:3e4eca7ead088f080af1476007decf010a240a1eb4e1e78c54fa32fbcc347084 https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.21%2B9/OpenJDK17U-jdk_aarch64_linux_hotspot_17.0.21_9.tar.gz /tmp/jdk.tar.gz
ADD --checksum=sha256:9f608c142e0e38a78957d0661ba1dfaf0b6fb37184cfaa4b9a0aa57af6184282 https://static.rust-lang.org/rustup/archive/1.29.3/aarch64-unknown-linux-gnu/rustup-init /tmp/rustup-init
COPY --from=amd64 /usr/lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu
COPY --from=amd64 /usr/lib64 /usr/lib64
RUN ln -s usr/lib64 /lib64

FROM $TARGETARCH

ARG NDK_VERSION
ARG BUILD_TOOLS_VERSION=36.0.0
ARG PLATFORM_VERSION=android-37.0
ARG ANDROID_TOOLS_URL=https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip
ARG ANDROID_TOOLS_SHA256=2d2d50857e4eb553af5a6dc3ad507a17adf43d115264b1afc116f95c92e5e258
ARG DEBIAN_FRONTEND=noninteractive

ENV JAVA_HOME=/opt/jdk
ENV ANDROID_HOME=/opt/android-sdk
ENV NDK_PATH=$ANDROID_HOME/ndk/$NDK_VERSION
ENV PATH=/root/.cargo/bin:$JAVA_HOME/bin:$ANDROID_HOME/cmdline-tools/latest/bin:$PATH
ENV GRADLE_COMMAND=./gradlew

SHELL ["/bin/bash", "-euo", "pipefail", "-c"]

RUN apt-get update -qq \
    && apt-get install -y -qq build-essential just curl unzip git ca-certificates > /dev/null

RUN mkdir -p "$JAVA_HOME" \
    && tar xzf /tmp/jdk.tar.gz -C "$JAVA_HOME" --strip-components=1 \
    && rm /tmp/jdk.tar.gz

ADD --checksum=sha256:$ANDROID_TOOLS_SHA256 $ANDROID_TOOLS_URL /tmp/tools.zip
RUN unzip -q /tmp/tools.zip -d /tmp \
    && mkdir -p "$ANDROID_HOME/cmdline-tools" \
    && mv /tmp/cmdline-tools "$ANDROID_HOME/cmdline-tools/latest" \
    && rm /tmp/tools.zip

RUN (yes || true) | sdkmanager --licenses > /dev/null \
    && sdkmanager --install "ndk;$NDK_VERSION" "build-tools;$BUILD_TOOLS_VERSION" "platforms;$PLATFORM_VERSION" platform-tools > /dev/null

RUN chmod +x /tmp/rustup-init \
    && /tmp/rustup-init -y --profile minimal --default-toolchain none \
    && rm /tmp/rustup-init

COPY rust-toolchain.toml /opt/wallet/
COPY android/reproducible/init.sh android/reproducible/versions.sh /opt/wallet/android/reproducible/
RUN /opt/wallet/android/reproducible/init.sh

COPY android/gradlew /opt/wallet/android/
COPY android/gradle/wrapper /opt/wallet/android/gradle/wrapper
RUN /opt/wallet/android/gradlew --version > /dev/null

COPY android/reproducible/entrypoint.sh /opt/wallet/android/reproducible/
ENTRYPOINT ["/opt/wallet/android/reproducible/entrypoint.sh"]
