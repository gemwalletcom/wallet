// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import func Gemstone.assetListRow
import func Gemstone.feeAmount
import enum Gemstone.FeeOption
import struct Gemstone.GemAssetItemRow
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemCustomFeeSession
import struct Gemstone.GemFeeAmount
import struct Gemstone.GemFeeAsset
import struct Gemstone.GemFeeOptionItem
import struct Gemstone.GemFeeRateRow
import struct Gemstone.GemFeeRateRows
import struct Gemstone.GemNetworkFeeScreen
import enum Gemstone.GemSelectAssetType
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit

public extension NetworkFeeSceneViewModel {
    static func mock(
        feeAsset: Asset = .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18),
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
        let rates = feeRates.map { rates -> GemFeeRateRows in
            var rates = rates
            rates.rows = rates.rows.map { row -> GemFeeRateRow in
                var row = row
                row.amount = row.fee.map(formatted)
                return row
            }
            return rates
        }
        let assets = feeAssets.map { GemFeeAsset.mock(asset: $0.asset.toGem(), row: $0.row) }
        return NetworkFeeSceneViewModel(
            screen: GemNetworkFeeScreen(
                fee: feeAmount.map(formatted),
                additionalFees: additionalFees.map { GemFeeOptionItem(option: $0.0, value: $0.1, amount: formatted($0.1)) },
                rates: rates,
                feeAsset: showsFeeAssets ? assets.first(where: { $0.asset.id == feeAsset.id.identifier }) : nil,
                feeAssets: assets,
                custom: rates.map {
                    GemCustomFeeSession(
                        feeAsset: feeAsset.toGem(),
                        input: "",
                        format: NumberInput.format(),
                        rows: $0,
                        loadedFee: feeAmount,
                        price: feeAssetPrice?.price,
                        currency: Currency.usd.toGem(),
                    )
                },
            ),
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
                data: AssetData.mock(asset: asset).toGem(),
                currency: Primitives.Currency.usd.toGem(),
                scope: .available,
                style: GemSelectAssetType.send.flow().rowStyle,
            ),
            isSelected: false,
        )
    }
}
