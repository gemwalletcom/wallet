// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.addressCopy
import func Gemstone.assetListRows
import struct Gemstone.GemAssetBalance
import struct Gemstone.GemAssetItemRow
import struct Gemstone.GemAssetListRowInput
import struct Gemstone.GemAssetRowStyle
import func Gemstone.walletAssetRows
import GemstonePrimitives
import Primitives

public struct ListAssetItemsViewModel {
    private let currency: Currency
    private let rowStyle: GemAssetRowStyle?

    public init(currency: Currency, rowStyle: GemAssetRowStyle? = nil) {
        self.currency = currency
        self.rowStyle = rowStyle
    }

    public func rows(_ assetDatas: [AssetData]) -> [GemAssetItemRow] {
        let inputs = assetDatas.map { $0.rowInput(currency: currency) }
        guard let rowStyle else {
            return walletAssetRows(inputs: inputs)
        }
        return assetListRows(inputs: inputs, style: rowStyle)
    }

    public func copyMessage(chain: Chain, address: String) -> String {
        CopyTypeViewModel(content: addressCopy(chain: chain.toGem(), address: address)).message
    }
}

public extension AssetData {
    func rowInput(currency: Currency) -> GemAssetListRowInput {
        GemAssetListRowInput(
            asset: asset.toGem(),
            balance: GemAssetBalance(balance, assetId: asset.id, isActive: metadata.isActive),
            scope: .total,
            price: price?.price,
            change: price?.priceChangePercentage24h,
            currency: currency.toGem(),
            isEnabled: metadata.isBalanceEnabled,
        )
    }
}
