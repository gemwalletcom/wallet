// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.SwapperSwapResult
import enum Gemstone.SwapperSwapStatus
import struct Gemstone.SwapperTransactionSwapMetadata

public extension SwapperSwapResult {
    static func mock(
        status: SwapperSwapStatus = .completed,
        metadata: SwapperTransactionSwapMetadata? = nil,
    ) -> SwapperSwapResult {
        SwapperSwapResult(
            status: status,
            metadata: metadata,
        )
    }
}
