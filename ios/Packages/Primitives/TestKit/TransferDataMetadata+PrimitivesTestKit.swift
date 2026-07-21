// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension TransferDataMetadata {
    static func mock(
        assetId: AssetId = .mock(),
        feeAssetId: AssetId = .mock(),
        assetBalance: Balance = .mock(),
        assetFeeBalance: Balance = .mock(),
        assetPrices: [AssetId: Price] = [:],
    ) -> TransferDataMetadata {
        TransferDataMetadata(
            assetId: assetId,
            feeAssetId: feeAssetId,
            assetBalance: assetBalance,
            assetFeeBalance: assetFeeBalance,
            assetPrices: assetPrices,
        )
    }
}
