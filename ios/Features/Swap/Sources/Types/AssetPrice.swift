// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemSwapValue
import Primitives

public struct AssetPriceValue {
    public let asset: Asset
    public let price: Price?

    public init(asset: Asset, price: Price?) {
        self.asset = asset
        self.price = price
    }
}

public extension AssetPriceValue {
    func swapValue(_ value: String) -> GemSwapValue {
        GemSwapValue(value: value, decimals: UInt32(asset.decimals), price: price?.price)
    }
}
