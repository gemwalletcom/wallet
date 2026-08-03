// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives

public final class PaymentSheetPresentableMock: PaymentSheetPresentable, @unchecked Sendable {
    public init() {}

    public var collectDataError: Error?
    public var selectedQuoteId: String?

    public private(set) var collectDataRequests: [PaymentDataCollectionRequest] = []
    public private(set) var quotesRequests: [PaymentQuotesRequest] = []

    public func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String {
        quotesRequests.append(request)
        guard let selectedQuoteId else {
            throw AnyError("no quote selected")
        }
        return selectedQuoteId
    }

    public func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String {
        collectDataRequests.append(request)
        if let collectDataError {
            throw collectDataError
        }
        return .empty
    }
}
