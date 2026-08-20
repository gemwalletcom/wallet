// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives

public struct PaymentServiceableMock: PaymentServiceable {
    private let onConfirm: (@Sendable (PaymentData, String) async throws -> PaymentOutcome)?

    public init(onConfirm: (@Sendable (PaymentData, String) async throws -> PaymentOutcome)? = nil) {
        self.onConfirm = onConfirm
    }

    public func getOptions(link _: PaymentLink, addresses _: [ChainAddress]) async throws -> PaymentOptions {
        throw AnyError("not mocked")
    }

    public func getQuoteData(quote _: PaymentQuote, addresses _: [ChainAddress]) async throws -> PaymentQuoteData {
        throw AnyError("not mocked")
    }

    public func confirm(payment: PaymentData, transactionHash: String) async throws -> PaymentOutcome {
        guard let onConfirm else {
            return PaymentOutcome(status: .succeeded, transactionId: .none)
        }
        return try await onConfirm(payment, transactionHash)
    }
}

public extension PaymentServiceable where Self == PaymentServiceableMock {
    static func mock(
        onConfirm: (@Sendable (PaymentData, String) async throws -> PaymentOutcome)? = nil,
    ) -> PaymentServiceableMock {
        PaymentServiceableMock(onConfirm: onConfirm)
    }
}
