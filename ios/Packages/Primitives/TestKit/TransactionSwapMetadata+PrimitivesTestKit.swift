// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension TransactionSwapMetadata {
    static func mock(
        fromAsset: AssetId = .mock(),
        fromValue: String = "0",
        toAsset: AssetId = .mock(.smartChain),
        toValue: String = "0",
        provider: String? = nil,
    ) -> TransactionSwapMetadata {
        TransactionSwapMetadata(
            fromAsset: fromAsset,
            fromValue: fromValue,
            toAsset: toAsset,
            toValue: toValue,
            provider: provider,
        )
    }
}
