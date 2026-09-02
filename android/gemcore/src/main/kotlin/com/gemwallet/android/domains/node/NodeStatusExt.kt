package com.gemwallet.android.domains.node

import com.gemwallet.android.model.NodeStatus

fun uniffi.gemstone.NodeStatus.toNodeStatus(url: String): NodeStatus = NodeStatus(
    url = url,
    chainId = chainId,
    blockNumber = latestBlockNumber,
    inSync = true,
    latency = latencyMs,
)
