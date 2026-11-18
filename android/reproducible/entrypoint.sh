#!/usr/bin/env bash
set -euo pipefail

mkdir /src
tar -xf - -C /src
cd /src/android

reproducible/build.sh "$1"

cd app/build/outputs
find . \( -name "*.apk" -o -name "*.aab" \) -exec cp --parents {} /outputs \;
