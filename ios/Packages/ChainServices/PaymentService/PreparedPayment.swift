// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct PreparedPayment: Sendable {
    public let quotes: PaymentQuotes
    public let quote: PaymentQuote
    public let actions: [PaymentAction]

    public init(quotes: PaymentQuotes, quote: PaymentQuote, actions: [PaymentAction]) {
        self.quotes = quotes
        self.quote = quote
        self.actions = actions
    }
}
