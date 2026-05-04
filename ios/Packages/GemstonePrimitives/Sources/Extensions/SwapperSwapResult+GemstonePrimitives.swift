// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.SwapperSwapResult {
    func map() throws -> Primitives.SwapResult {
        try Primitives.SwapResult(
            status: status.map(),
            metadata: metadata?.map(),
        )
    }
}

public extension Gemstone.SwapperSwapStatus {
    func map() -> Primitives.SwapStatus {
        switch self {
        case .pending: .pending
        case .completed: .completed
        case .failed: .failed
        }
    }
}

public extension Gemstone.SwapperTransactionSwapMetadata {
    func map() throws -> Primitives.TransactionSwapMetadata {
        try Primitives.TransactionSwapMetadata(
            fromAsset: AssetId(id: fromAsset),
            fromValue: fromValue,
            toAsset: AssetId(id: toAsset),
            toValue: toValue,
            provider: provider,
        )
    }
}
