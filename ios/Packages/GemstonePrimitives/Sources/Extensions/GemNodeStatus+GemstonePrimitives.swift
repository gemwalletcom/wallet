// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Gemstone
import Primitives

public extension Gemstone.NodeStatus {
    func map() -> Primitives.NodeStatus {
        Primitives.NodeStatus(
            chainId: chainId,
            latestBlockNumber: BigInt(latestBlockNumber),
            latency: Latency.from(milliseconds: Double(latencyMs)),
        )
    }
}
