package com.gemwallet.android.testkit

import uniffi.gemstone.GemNodeStatusState
import uniffi.gemstone.Latency
import uniffi.gemstone.LatencyType

fun mockGemNodeStatusState(latestBlockNumber: ULong = 1UL) = GemNodeStatusState.Result(
    latestBlockNumber = latestBlockNumber,
    latency = Latency(LatencyType.FAST, 10.0),
)
