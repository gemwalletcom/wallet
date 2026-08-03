// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct PaymentData: Hashable, Equatable, Sendable {
    public let provider: PaymentProviderName
    public let quotes: PaymentQuotes
    public let quote: PaymentQuote

    public init(provider: PaymentProviderName, quotes: PaymentQuotes, quote: PaymentQuote) {
        self.provider = provider
        self.quotes = quotes
        self.quote = quote
    }

    public var merchant: PaymentMerchant {
        quotes.merchant
    }

    public var price: PaymentPrice? {
        quotes.price
    }

    public var expiresAt: Date? {
        quotes.expiresAt
    }
}
