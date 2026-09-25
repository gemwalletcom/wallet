// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import func Gemstone.feeAmount
import enum Gemstone.FeeOption
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemFeeAmount
import struct Gemstone.GemFeeOptionItem
import struct Gemstone.GemFeeRateRow
import struct Gemstone.GemFeeRateRows
import GemstonePrimitives
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
        additionalFees: [(FeeOption, BigInt)] = [],
        feeAssets: [FeeAssetItem] = [],
        showsFeeAssets: Bool = false,
        onSelect: (@MainActor (GemConfirmFeeSelection) -> Void)? = nil,
        onSelectFeeAsset: (@MainActor (AssetId) -> Void)? = nil,
    ) -> NetworkFeeSceneViewModel {
        let formatted = { (value: BigInt) -> GemFeeAmount in
            Gemstone.feeAmount(asset: feeAsset.toGem(), value: value, price: feeAssetPrice?.price, currency: Currency.usd.toGem())
        }
        let feeRates = feeRates.map { rates -> GemFeeRateRows in
            var rates = rates
            rates.rows = rates.rows.map { row -> GemFeeRateRow in
                var row = row
                row.amount = row.fee.map(formatted)
                return row
            }
            return rates
        }
        return NetworkFeeSceneViewModel(
            feeAsset: feeAsset,
            currency: .usd,
            selection: selection,
            feeRates: feeRates,
            feeAssetPrice: feeAssetPrice,
            feeAmount: feeAmount,
            fee: feeAmount.map(formatted),
            additionalFees: additionalFees.map { GemFeeOptionItem(option: $0.0, value: $0.1, amount: formatted($0.1)) },
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
