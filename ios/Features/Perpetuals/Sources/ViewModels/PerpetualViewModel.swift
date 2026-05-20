// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct PerpetualViewModel {
    public let perpetual: Perpetual
    private let marketValueFormatter: CurrencyFormatter
    private let priceFormatter: CurrencyFormatter
    private let percentFormatter = PercentFormatter.signed

    public init(perpetual: Perpetual, currencyStyle: CurrencyFormatterType = .abbreviated) {
        self.perpetual = perpetual
        marketValueFormatter = CurrencyFormatter(type: currencyStyle, currencyCode: Currency.usd.rawValue)
        priceFormatter = CurrencyFormatter(type: .currency, currencyCode: Currency.usd.rawValue)
    }

    public var name: String {
        perpetual.name
    }

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: perpetual.assetId).assetImage
    }

    public var volumeField: ListItemField {
        ListItemField(title: Localized.Markets.dailyVolume, value: marketValueFormatter.string(perpetual.volume24h))
    }

    public var openInterestField: ListItemField {
        ListItemField(title: Localized.Info.OpenInterest.title, value: marketValueFormatter.string(perpetual.openInterest))
    }

    public var fundingRateField: ListItemField {
        let annualized = perpetual.funding * 24 * 365
        return ListItemField(title: Localized.Info.FundingApr.title, value: percentFormatter.string(annualized))
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
