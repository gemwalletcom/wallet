// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import GemstonePrimitives
import class Gemstone.GemPerpetual
import struct Gemstone.GemPerpetualMarketRow
import enum Gemstone.GemPerpetualInfoRow
import func Gemstone.perpetualMarketRow
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct PerpetualViewModel {
    public let perpetual: Perpetual
    public let row: GemPerpetualMarketRow
    private let priceFormatter: CurrencyFormatter
    private let percentFormatter = PercentFormatter.signed

    public init(
        perpetual: Perpetual,
        priceFormatter: CurrencyFormatter = .usd,
    ) {
        self.perpetual = perpetual
        self.priceFormatter = priceFormatter
        row = perpetualMarketRow(perpetual: perpetual.toGem())
    }

    public var name: String {
        row.title
    }

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: perpetual.assetId).assetImage
    }

    public func infoField(for infoRow: GemPerpetualInfoRow) -> ListItemField {
        let value = switch infoRow {
        case .dailyVolume: row.volume24h.text()
        case .openInterest: row.openInterest.text()
        case .fundingRate: percentFormatter.string(GemPerpetual(provider: perpetual.provider.toGem()).fundingApr(funding: perpetual.funding))
        }
        return ListItemField(title: infoRow.title, value: value)
    }

    public var priceText: String {
        priceFormatter.string(perpetual.price)
    }

    public var priceChangeText: String {
        percentFormatter.string(perpetual.pricePercentChange24h)
    }

    public var priceChangeTextColor: Color {
        PriceChangeColor.color(for: perpetual.pricePercentChange24h)
    }
}
