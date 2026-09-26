// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.addressCopy
import func Gemstone.assetListRows
import struct Gemstone.GemAssetItemRow
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
        let assets = assetDatas.map { $0.toGem() }
        guard let rowStyle else {
            return walletAssetRows(assets: assets, currency: currency.toGem())
        }
        return assetListRows(assets: assets, currency: currency.toGem(), style: rowStyle)
    }

    public func copyMessage(chain: Chain, address: String) -> String {
        CopyTypeViewModel(content: addressCopy(chain: chain.toGem(), address: address)).message
    }
}
