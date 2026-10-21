#!/usr/bin/env bash
set -euo pipefail

seconds="${1:-600}"
cd "$(dirname "$0")"

targets=(solana_transaction ton_boc protobuf external_input)
host="$(rustc -vV | sed -n 's/^host: //p')"

cp ../Cargo.lock Cargo.lock
RUSTFLAGS="--cfg fuzzing -Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares -Cdebug-assertions -Coverflow-checks" \
    cargo build --release --bins --target "$host"

for target in "${targets[@]}"; do
    mkdir -p "corpus/$target" "artifacts/$target"
done

seed_base64() {
    local target="$1" pattern="$2"
    local index=0
    grep -rhoE "$pattern" ../crates ../gemstone/src | sort -u | while read -r value; do
        index=$((index + 1))
        echo "$value" | tr '_-' '/+' | base64 -d > "corpus/$target/seed-$index" 2>/dev/null || rm -f "corpus/$target/seed-$index"
    done
}

seed_base64 solana_transaction '(AQ|Ag|Aw|gA)[A-Za-z0-9+/]{200,}={0,2}'
seed_base64 ton_boc 'te6cc[A-Za-z0-9+/_-]+={0,2}'

index=0
grep -rhoE '"(bitcoin|ethereum|solana|ton|xrp|ripple|xrpl|dogecoin|litecoin|bitcoincash|wc|https):[^"]+"' ../crates/payment/src | tr -d '"' | sort -u | while read -r value; do
    index=$((index + 1))
    printf '\000\000\000%s' "$value" > "corpus/external_input/seed-$index"
done

for target in "${targets[@]}"; do
    "target/$host/release/$target" "corpus/$target" \
        -artifact_prefix="artifacts/$target/" -max_total_time="$seconds" -timeout=10 \
        -fork=1 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 \
        > "artifacts/$target.log" 2>&1 &
done
wait

for target in "${targets[@]}"; do
    echo "$target: $(find "artifacts/$target" -type f | wc -l | tr -d ' ') findings, $(find "corpus/$target" -type f | wc -l | tr -d ' ') corpus inputs"
done
