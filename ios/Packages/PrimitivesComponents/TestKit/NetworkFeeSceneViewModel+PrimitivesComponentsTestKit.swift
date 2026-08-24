// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension NetworkFeeSceneViewModel {
    static func mock(
        feeAsset: Asset = .mockEthereum(),
        currency: Currency = .usd,
        selection: FeeSelection = .preset(.normal),
        rates: [FeeRate] = [],
        feeAssetPrice: Price? = nil,
        feeAmount: BigInt? = nil,
        feeAssets: [AssetData] = [],
        onSelect: (@MainActor (FeeSelection) -> Void)? = nil,
        onSelectFeeAsset: (@MainActor (AssetId) -> Void)? = nil,
    ) -> NetworkFeeSceneViewModel {
        NetworkFeeSceneViewModel(
            feeAsset: feeAsset,
            currency: currency,
            selection: selection,
            rates: rates,
            feeAssetPrice: feeAssetPrice,
            feeAmount: feeAmount,
            feeAssets: feeAssets,
            onSelect: onSelect,
            onSelectFeeAsset: onSelectFeeAsset,
        )
    }
}
