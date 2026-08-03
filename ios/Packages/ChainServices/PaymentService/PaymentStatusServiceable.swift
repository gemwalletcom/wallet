// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol PaymentStatusServiceable: Sendable {
    func hasStatus(provider: PaymentProviderName) -> Bool
    func getPaymentStatus(provider: PaymentProviderName, paymentId: String) async throws -> PaymentOutcome
}
