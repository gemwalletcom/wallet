// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol PaymentSheetPresentable: Sendable {
    func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String
    func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String
}
