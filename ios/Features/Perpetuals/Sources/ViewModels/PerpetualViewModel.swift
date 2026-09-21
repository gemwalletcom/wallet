// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemPerpetualMarketRow
import func Gemstone.perpetualMarketRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct PerpetualViewModel {
    public let perpetual: Perpetual
    public let row: GemPerpetualMarketRow

    public init(perpetual: Perpetual, asset: Asset) {
        self.perpetual = perpetual
        row = perpetualMarketRow(perpetual: perpetual.toGem(), asset: asset.toGem())
    }

    public var name: String {
        row.title
    }

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: perpetual.assetId).assetImage
    }

    public var priceText: String {
        row.price.price?.text() ?? .empty
    }

    public var priceChangeText: String {
        row.price.change?.text() ?? .empty
    }

    public var priceChangeTextColor: Color {
        row.price.change?.tone.color ?? Colors.gray
    }
}
