// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import class Gemstone.GemFeeService
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
        feeAssets: [FeeAssetItem] = [],
        feeService: GemFeeService = GemFeeService(),
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
            feeService: feeService,
            onSelect: onSelect,
            onSelectFeeAsset: onSelectFeeAsset,
        )
    }
}

public extension FeeAssetItem {
    static func mock(
        asset: Asset = .mockEthereum(),
        balance: Balance = .zero,
        price: Price? = nil,
        currency: Currency = .usd,
        isSelected: Bool = false,
    ) -> FeeAssetItem {
        FeeAssetItem(asset: asset, balance: balance, price: price, currency: currency, isSelected: isSelected)
    }
}
