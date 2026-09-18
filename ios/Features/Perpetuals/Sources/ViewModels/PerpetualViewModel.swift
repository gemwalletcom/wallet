// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import GemstonePrimitives
import struct Gemstone.GemPerpetualMarketRow
import func Gemstone.perpetualMarketRow
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import func Gemstone.valueTone

public struct PerpetualViewModel {
    public let perpetual: Perpetual
    public let row: GemPerpetualMarketRow
    private let priceFormatter: CurrencyFormatter
    private let percentFormatter = PercentFormatter.signed

    public init(
        perpetual: Perpetual,
        asset: Asset,
        priceFormatter: CurrencyFormatter = .usd,
    ) {
        self.perpetual = perpetual
        self.priceFormatter = priceFormatter
        row = perpetualMarketRow(perpetual: perpetual.toGem(), asset: asset.toGem())
    }

    public var name: String {
        row.title
    }

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: perpetual.assetId).assetImage
    }

    public var priceText: String {
        priceFormatter.string(perpetual.price)
    }

    public var priceChangeText: String {
        percentFormatter.string(perpetual.pricePercentChange24h)
    }

    public var priceChangeTextColor: Color {
        valueTone(value: perpetual.pricePercentChange24h).color
    }
}
