#!/bin/zsh
# usage: run.sh <label> <base_url> <scenarios file> [pid]
# scenario line: name|METHOD|path|body|auth   (auth: none | device | device-wallet | bearer:<secret>)
# env: DURATION (30s), THREADS (8), CONNECTIONS (comma list, default 64,256), DEVICE_SEED (hex, for device auth), WALLET_ID
set -euo pipefail
label=$1; base=$2; scenarios=$3; pid=${4:-}
duration=${DURATION:-30s}; threads=${THREADS:-8}; connections=${CONNECTIONS:-64,256}
here=$(cd "$(dirname "$0")" && pwd)
out="$here/results/$label.md"
header_for() {
  local method=$1 path=$2 auth=$3
  case $auth in
    none) echo "";;
    bearer:*) echo "Authorization: Bearer ${auth#bearer:}";;
    device) echo "Authorization: $(cargo run -q -p api --example device_header -- "$DEVICE_SEED" header "$method" "${path%%\?*}")";;
    device-wallet) echo "Authorization: $(cargo run -q -p api --example device_header -- "$DEVICE_SEED" header "$method" "${path%%\?*}" "$WALLET_ID")";;
  esac
}
sample_rss() {
  [ -n "$pid" ] && ps -o rss= -p "$pid" | tr -d ' ' || echo 0
}
{
  echo "# $label"
  echo
  echo "base=$base duration=$duration threads=$threads date=$(date -u +%FT%TZ)"
  echo
  echo "| scenario | conns | req/s | p50 ms | p90 ms | p99 ms | max ms | non-2xx | errors | rss MB |"
  echo "|---|---|---|---|---|---|---|---|---|---|"
} > "$out"
while IFS='|' read -r name method path body auth; do
  [ -z "$name" ] && continue
  [[ $name == \#* ]] && continue
  for conns in ${(s:,:)connections}; do
    WRK_METHOD=$method WRK_PATH=$path WRK_BODY=$body WRK_HEADERS=$(header_for "$method" "$path" "$auth") \
      wrk -t"$threads" -c"$conns" -d2s -s "$here/request.lua" "$base" > /dev/null
    line=$(WRK_METHOD=$method WRK_PATH=$path WRK_BODY=$body WRK_HEADERS=$(header_for "$method" "$path" "$auth") \
      wrk -t"$threads" -c"$conns" -d"$duration" --latency -s "$here/request.lua" "$base" | grep '^RESULT')
    rss=$(sample_rss)
    eval "$(echo "$line" | sed 's/^RESULT //; s/\([a-z0-9_]*\)=\([^ ]*\)/\1=\2;/g')"
    printf "| %s | %s | %.0f | %.2f | %.2f | %.2f | %.1f | %s | %s | %.0f |\n" "$name" "$conns" "$rps" "$((p50_us))e-3" "$((p90_us))e-3" "$((p99_us))e-3" "$((max_us))e-3" "$non2xx" "$errors" "$((rss))e-3" >> "$out"
  done
done < "$scenarios"
cat "$out"
