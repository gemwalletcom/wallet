// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import struct Gemstone.GemFiatQuoteRow
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct FiatQuoteViewModel {
    let row: GemFiatQuoteRow
    let isSelected: Bool

    private let asset: Asset
    private let formatter: CurrencyFormatter

    init(
        asset: Asset,
        row: GemFiatQuoteRow,
        isSelected: Bool = false,
        formatter: CurrencyFormatter,
    ) {
        self.asset = asset
        self.row = row
        self.isSelected = isSelected
        self.formatter = formatter
    }

    var title: String {
        row.providerName
    }

    var amountText: String {
        NumericFormatter().string(row.cryptoAmount, symbol: asset.symbol)
    }

    var rateText: String {
        guard let rate = row.rate else { return "" }
        return rate.text(formattedValue: formatter.string(rate.value))
    }
}

extension FiatQuoteViewModel: Identifiable {
    var id: String {
        "\(asset.id.identifier)\(row.provider.map().rawValue)\(row.cryptoAmount)"
    }
}

// MARK: - SimpleListItemViewable

extension FiatQuoteViewModel: SimpleListItemViewable {
    var titleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    var assetImage: AssetImage {
        AssetImage(
            placeholder: row.provider.map().image,
            chainPlaceholder: isSelected ? Images.Wallets.selected : nil,
        )
    }

    var subtitle: String? {
        amountText
    }

    var subtitleExtra: String? {
        formatter.string(row.fiatAmount)
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
