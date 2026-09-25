// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import func Gemstone.assetListRow
import func Gemstone.feeAmount
import enum Gemstone.FeeOption
import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemAssetListRowInput
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemFeeAmount
import struct Gemstone.GemFeeOptionItem
import struct Gemstone.GemFeeRateRow
import struct Gemstone.GemFeeRateRows
import enum Gemstone.GemSelectAssetType
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension NetworkFeeSceneViewModel {
    static func mock(
        feeAsset: Asset = .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
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
            selectedFeeAsset: feeAssets.first(where: { $0.asset.id == feeAsset.id }),
            showsFeeAssets: showsFeeAssets,
            onSelect: onSelect,
            onSelectFeeAsset: onSelectFeeAsset,
        )
    }
}

public extension FeeAssetItem {
    static func mock(asset: Asset = .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18)) -> FeeAssetItem {
        FeeAssetItem(
            asset: asset,
            row: assetListRow(
                input: GemAssetListRowInput(
                    asset: asset.toGem(),
                    balance: .mock(assetId: asset.id.identifier),
                    scope: .available,
                    price: nil,
                    change: nil,
                    currency: Primitives.Currency.usd.toGem(),
                    isEnabled: true,
                ),
                style: GemSelectAssetType.send.flow().rowStyle,
            ),
            isSelected: false,
        )
    }
}
