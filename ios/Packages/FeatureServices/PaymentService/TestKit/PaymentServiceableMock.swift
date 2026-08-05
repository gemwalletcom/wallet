// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives
import PrimitivesTestKit

public actor PaymentServiceableMock: PaymentServiceable {
    private var options: [PaymentOptions]
    private let actions: [PaymentAction]
    private let isRelayed: Bool
    private let confirmOutcome: PaymentOutcome
    private let confirmError: (any Error)?
    private let statusOutcome: PaymentOutcome

    public private(set) var confirmedResults: [[String]] = []
    public private(set) var requestedQuotes: [PaymentQuote] = []

    public init(
        options: [PaymentOptions],
        actions: [PaymentAction] = [],
        isRelayed: Bool = true,
        confirmOutcome: PaymentOutcome = .mock(),
        confirmError: (any Error)? = .none,
        statusOutcome: PaymentOutcome = .mock(),
    ) {
        self.options = options
        self.actions = actions
        self.isRelayed = isRelayed
        self.confirmOutcome = confirmOutcome
        self.confirmError = confirmError
        self.statusOutcome = statusOutcome
    }

    public func getPaymentOptions(link _: PaymentLink, wallet _: Wallet) async throws -> PaymentOptions {
        guard !options.isEmpty else {
            throw AnyError("Unexpected payment options request")
        }
        return options.removeFirst()
    }

    public func getPreparedPayment(provider _: PaymentProviderName, quotes: PaymentQuotes, quote: PaymentQuote, wallet _: Wallet) async throws -> PreparedPayment {
        requestedQuotes.append(quote)
        return PreparedPayment(
            quotes: quotes,
            quote: quote,
            actions: actions,
            isRelayed: isRelayed,
        )
    }

    public func confirmPayment(provider _: PaymentProviderName, quote _: PaymentQuote, actionResults: [String]) async throws -> PaymentOutcome {
        confirmedResults.append(actionResults)
        if let confirmError {
            throw confirmError
        }
        return confirmOutcome
    }

    public nonisolated func hasStatus(provider _: PaymentProviderName) -> Bool {
        true
    }

    public func getPaymentStatus(provider _: PaymentProviderName, paymentId _: String) async throws -> PaymentOutcome {
        statusOutcome
    }
}
