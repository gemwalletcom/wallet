// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct PaymentData: Hashable, Sendable {
    public let quote: PaymentQuote
    public let merchant: PaymentMerchant

    public init(quote: PaymentQuote, merchant: PaymentMerchant) {
        self.quote = quote
        self.merchant = merchant
    }
}
