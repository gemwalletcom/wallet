FROM ubuntu@sha256:68ee22004a9109b44973313b02ff7b1de250f1de0f5ecb628d73f579aaff33ac

ARG NDK_VERSION
ARG JDK_URL=https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.21%2B9/OpenJDK17U-jdk_x64_linux_hotspot_17.0.21_9.tar.gz
ARG JDK_SHA256=8b9b611d917e3dfd8419d3566b5949226f69b052fed7e109c1368add1dfe8711
ARG ANDROID_TOOLS_URL=https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip
ARG ANDROID_TOOLS_SHA256=2d2d50857e4eb553af5a6dc3ad507a17adf43d115264b1afc116f95c92e5e258
ARG RUSTUP_URL=https://static.rust-lang.org/rustup/archive/1.29.3/x86_64-unknown-linux-gnu/rustup-init
ARG RUSTUP_SHA256=b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55
ARG DEBIAN_FRONTEND=noninteractive

ENV JAVA_HOME=/opt/jdk
ENV ANDROID_HOME=/opt/android-sdk
ENV NDK_PATH=$ANDROID_HOME/ndk/$NDK_VERSION
ENV PATH=/root/.cargo/bin:$JAVA_HOME/bin:$ANDROID_HOME/cmdline-tools/latest/bin:$PATH
ENV GRADLE_COMMAND=./gradlew

SHELL ["/bin/bash", "-euo", "pipefail", "-c"]

RUN apt-get update -qq \
    && apt-get install -y -qq build-essential just curl unzip git ca-certificates > /dev/null

ADD --checksum=sha256:$JDK_SHA256 $JDK_URL /tmp/jdk.tar.gz
RUN mkdir -p "$JAVA_HOME" \
    && tar xzf /tmp/jdk.tar.gz -C "$JAVA_HOME" --strip-components=1 \
    && rm /tmp/jdk.tar.gz

ADD --checksum=sha256:$ANDROID_TOOLS_SHA256 $ANDROID_TOOLS_URL /tmp/tools.zip
RUN unzip -q /tmp/tools.zip -d /tmp \
    && mkdir -p "$ANDROID_HOME/cmdline-tools" \
    && mv /tmp/cmdline-tools "$ANDROID_HOME/cmdline-tools/latest" \
    && rm /tmp/tools.zip

RUN (yes || true) | sdkmanager --licenses > /dev/null \
    && sdkmanager --install "ndk;$NDK_VERSION" > /dev/null

ADD --checksum=sha256:$RUSTUP_SHA256 $RUSTUP_URL /tmp/rustup-init
RUN chmod +x /tmp/rustup-init \
    && /tmp/rustup-init -y --profile minimal --default-toolchain none \
    && rm /tmp/rustup-init

COPY rust-toolchain.toml /opt/wallet/
COPY android/reproducible/init.sh android/reproducible/versions.sh /opt/wallet/android/reproducible/
RUN /opt/wallet/android/reproducible/init.sh

COPY android/reproducible/entrypoint.sh /opt/wallet/android/reproducible/
ENTRYPOINT ["/opt/wallet/android/reproducible/entrypoint.sh"]
