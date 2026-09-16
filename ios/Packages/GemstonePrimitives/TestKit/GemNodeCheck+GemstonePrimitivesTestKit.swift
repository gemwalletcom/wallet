// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemNodeCheck
import struct Gemstone.Latency

public extension GemNodeCheck {
    static func mock(url: String) -> GemNodeCheck {
        GemNodeCheck(url: url, chainId: "1", latestBlockNumber: 21_000_000, isInSync: true, latency: Latency(latencyType: .fast, value: 12))
    }
}
