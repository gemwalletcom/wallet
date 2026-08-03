// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol PaymentServiceable: PaymentStatusServiceable {
    func getPaymentOptions(link: PaymentLink, wallet: Wallet) async throws -> PaymentOptions
    func getPreparedPayment(provider: PaymentProviderName, quotes: PaymentQuotes, quote: PaymentQuote, wallet: Wallet) async throws -> PreparedPayment
    func confirmPayment(provider: PaymentProviderName, quote: PaymentQuote, actionResults: [String]) async throws -> PaymentOutcome
    func cancelPayment(provider: PaymentProviderName, paymentId: String) async throws
}
