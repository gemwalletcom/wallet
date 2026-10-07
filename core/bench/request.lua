local method = os.getenv("WRK_METHOD") or "GET"
local path = os.getenv("WRK_PATH") or "/"
local body = os.getenv("WRK_BODY") or ""
local headers = {}
for line in (os.getenv("WRK_HEADERS") or ""):gmatch("[^\n]+") do
  local name, value = line:match("^%s*([^:]+):%s*(.*)$")
  if name then headers[name] = value end
end
if body ~= "" and headers["Content-Type"] == nil then headers["Content-Type"] = "application/json" end

request = function()
  return wrk.format(method, path, headers, body)
end

done = function(summary, latency, requests)
  io.write(string.format("RESULT requests=%d duration_us=%d rps=%.1f p50_us=%d p90_us=%d p99_us=%d max_us=%d errors=%d non2xx=%d\n",
    summary.requests, summary.duration, summary.requests / (summary.duration / 1e6),
    latency:percentile(50), latency:percentile(90), latency:percentile(99), latency.max,
    summary.errors.connect + summary.errors.read + summary.errors.write + summary.errors.timeout, summary.errors.status))
end
