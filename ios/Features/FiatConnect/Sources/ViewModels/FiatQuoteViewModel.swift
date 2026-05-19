// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct FiatQuoteViewModel {
    let quote: FiatQuote
    let assetPrice: Double?
    let isSelected: Bool

    private let asset: Asset
    private let formatter: CurrencyFormatter

    init(
        asset: Asset,
        quote: FiatQuote,
        assetPrice: Double? = nil,
        isSelected: Bool = false,
        formatter: CurrencyFormatter,
    ) {
        self.asset = asset
        self.quote = quote
        self.assetPrice = assetPrice
        self.isSelected = isSelected
        self.formatter = formatter
    }

    var title: String {
        quote.provider.name
    }

    var amountText: String {
        NumericFormatter().string(quote.cryptoAmount, symbol: asset.symbol)
    }

    var rateText: String {
        let amount = quote.fiatAmount / quote.cryptoAmount
        return formatter.string(amount)
    }

    private var priceText: String {
        guard let assetPrice, assetPrice > 0 else {
            return formatter.string(quote.fiatAmount)
        }
        return formatter.string(assetPrice * quote.cryptoAmount)
    }
}

extension FiatQuoteViewModel: Identifiable {
    var id: String {
        "\(asset.id.identifier)\(quote.provider.id)\(quote.cryptoAmount)"
    }
}

// MARK: - SimpleListItemViewable

extension FiatQuoteViewModel: SimpleListItemViewable {
    var titleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    var assetImage: AssetImage {
        AssetImage(
            placeholder: quote.provider.image,
            chainPlaceholder: isSelected ? Images.Wallets.selected : nil,
        )
    }

    var subtitle: String? {
        amountText
    }

    var subtitleExtra: String? {
        switch quote.type {
        case .buy: priceText
        case .sell: formatter.string(quote.fiatAmount)
        }
    }

    var subtitleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    var subtitleStyleExtra: TextStyle {
        TextStyle(font: .footnote, color: Colors.gray)
    }
}

// MARK: - Hashable

extension FiatQuoteViewModel: Hashable {
    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }
}
