#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
cp ../Cargo.lock Cargo.lock
if [[ "$(uname)" == Darwin ]]; then
    export RUSTFLAGS="-Clink-arg=-undefined -Clink-arg=dynamic_lookup"
fi
cargo fuzz build -s none -a

seed() {
    mkdir -p "corpus/$1"
    grep -rhoE "$2" ../crates | sort -u | while read -r value; do
        $3 <<< "$value" > "corpus/$1/seed-$(cksum <<< "$value" | cut -d ' ' -f 1)" 2>/dev/null || true
    done
}
seed solana_transaction '(AQ|Ag|Aw|gA)[A-Za-z0-9+/]{200,}={0,2}' 'base64 -d'
seed ton_boc 'te6cc[A-Za-z0-9+/]+={0,2}' 'base64 -d'
seed payment_url '(bitcoin|ethereum|solana|ton|xrp|wc):[^" ]+' cat

mkdir -p artifacts
for target in $(cargo fuzz list); do
    cargo fuzz run -s none -a "$target" -- -max_total_time="${1:-600}" -timeout=10 -fork=1 -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 > "artifacts/$target.log" 2>&1 &
done
wait

for target in $(cargo fuzz list); do
    echo "$target: $(find "artifacts/$target" -type f 2>/dev/null | wc -l | tr -d ' ') findings"
done
