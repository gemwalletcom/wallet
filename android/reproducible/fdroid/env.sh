#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export SKIP_SIGN="${SKIP_SIGN:-true}"
export REPRO_BUILD_HOME_PREFIX="${REPRO_BUILD_HOME_PREFIX:-FDROID_HOME}"

source "$script_dir/../env.sh"
