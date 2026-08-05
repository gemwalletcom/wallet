// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct PreparedPayment: Sendable {
    public let quotes: PaymentQuotes
    public let quote: PaymentQuote
    public let actions: [PaymentAction]
    public let isRelayed: Bool

    public init(quotes: PaymentQuotes, quote: PaymentQuote, actions: [PaymentAction], isRelayed: Bool) {
        self.quotes = quotes
        self.quote = quote
        self.actions = actions
        self.isRelayed = isRelayed
    }
}
