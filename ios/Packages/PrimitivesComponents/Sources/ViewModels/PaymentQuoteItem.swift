// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import Primitives
import Style
import SwiftUI

public struct PaymentQuoteItem: Identifiable, Sendable {
    public let quote: PaymentQuote

    private let assetData: AssetData?
    private let formatter: ValueFormatter

    public init(
        quote: PaymentQuote,
        assetData: AssetData? = .none,
        formatter: ValueFormatter,
    ) {
        self.quote = quote
        self.assetData = assetData
        self.formatter = formatter
    }

    public var id: String {
        quote.id
    }

    public var amountText: String {
        guard let value = BigInt(quote.amount.value) else {
            return quote.amount.symbol
        }
        return formatter.string(value, decimals: quote.amount.decimals.asInt, currency: quote.amount.symbol)
    }
}

// MARK: - ListAssetItemViewable

extension PaymentQuoteItem: ListAssetItemViewable {
    public var name: String {
        assetData?.asset.name ?? quote.amount.symbol
    }

    public var symbol: String? {
        .none
    }

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: quote.amount.assetId).assetImage
    }

    public var subtitleView: ListAssetItemSubtitleView {
        .type(TextValue(text: quote.amount.assetId.chain.networkName, style: .calloutSecondary))
    }

    public var rightView: ListAssetItemRightView {
        .balance(
            balance: TextValue(text: amountText, style: TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)),
            totalFiat: TextValue(text: balanceText, style: TextStyle(font: .footnote, color: Colors.gray)),
        )
    }

    public var action: ((ListAssetItemAction) -> Void)? {
        get { .none }
        set {}
    }
}

// MARK: - Hashable

extension PaymentQuoteItem: Hashable {
    public static func == (lhs: PaymentQuoteItem, rhs: PaymentQuoteItem) -> Bool {
        lhs.id == rhs.id
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }
}

// MARK: - Private

extension PaymentQuoteItem {
    private var balanceText: String {
        guard let assetData else {
            return .empty
        }
        return formatter.string(assetData.balance.available, decimals: assetData.asset.decimals.asInt, currency: assetData.asset.symbol)
    }
}
