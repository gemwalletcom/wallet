// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PaymentQuoteItem: Identifiable, Sendable {
    let quote: PaymentQuote
    let asset: Asset

    private let balance: String
    private let formatter: ValueFormatter

    init(quote: PaymentQuote, asset: Asset, balance: String, formatter: ValueFormatter) {
        self.quote = quote
        self.asset = asset
        self.balance = balance
        self.formatter = formatter
    }

    var id: String { quote.id }

    var amountText: String {
        guard let value = BigInt(quote.value) else {
            return asset.symbol
        }
        return formatter.string(value, decimals: asset.decimals.asInt, currency: asset.symbol)
    }

    var assetImage: AssetImage {
        AssetIdViewModel(assetId: quote.assetId).assetImage
    }
}

// MARK: - ListAssetItemViewable

extension PaymentQuoteItem: ListAssetItemViewable {
    var name: String { asset.name }

    var symbol: String? { .none }

    var subtitleView: ListAssetItemSubtitleView {
        .type(TextValue(text: quote.assetId.chain.networkName, style: .calloutSecondary))
    }

    var rightView: ListAssetItemRightView {
        .balance(
            balance: TextValue(text: amountText, style: TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)),
            totalFiat: TextValue(text: balance, style: .footnote),
        )
    }

    var action: ((ListAssetItemAction) -> Void)? {
        get { .none }
        set {}
    }
}

// MARK: - Hashable

extension PaymentQuoteItem: Hashable {
    static func == (lhs: PaymentQuoteItem, rhs: PaymentQuoteItem) -> Bool {
        lhs.id == rhs.id
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }
}
