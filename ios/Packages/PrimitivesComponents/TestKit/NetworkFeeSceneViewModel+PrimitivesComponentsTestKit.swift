// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemFeeOptionItem
import struct Gemstone.GemFeeRateRows
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension NetworkFeeSceneViewModel {
    static func mock(
        feeAsset: Asset = .mockEthereum(),
        selection: GemConfirmFeeSelection = .priority(priority: .normal),
        feeRates: GemFeeRateRows? = nil,
        feeAssetPrice: Price? = nil,
        feeAmount: BigInt? = nil,
        additionalFees: [GemFeeOptionItem] = [],
        feeAssets: [FeeAssetItem] = [],
        showsFeeAssets: Bool = false,
        onSelect: (@MainActor (GemConfirmFeeSelection) -> Void)? = nil,
        onSelectFeeAsset: (@MainActor (AssetId) -> Void)? = nil,
    ) -> NetworkFeeSceneViewModel {
        NetworkFeeSceneViewModel(
            feeAsset: feeAsset,
            currency: .usd,
            selection: selection,
            feeRates: feeRates,
            feeAssetPrice: feeAssetPrice,
            feeAmount: feeAmount,
            additionalFees: additionalFees,
            feeAssets: feeAssets,
            showsFeeAssets: showsFeeAssets,
            onSelect: onSelect,
            onSelectFeeAsset: onSelectFeeAsset,
        )
    }
}

public extension FeeAssetItem {
    static func mock(asset: Asset = .mockEthereum()) -> FeeAssetItem {
        FeeAssetItem(asset: asset, balance: .zero, price: nil, currency: .usd, isSelected: false)
    }
}
