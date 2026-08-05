// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives
import PrimitivesTestKit

public actor PaymentStatusServiceableMock: PaymentStatusServiceable {
    private let result: PaymentOutcome
    private let providerHasStatus: Bool
    private var requestedPaymentIds: [String] = []

    public init(result: PaymentOutcome = .mock(), providerHasStatus: Bool = true) {
        self.result = result
        self.providerHasStatus = providerHasStatus
    }

    public nonisolated func hasStatus(provider _: PaymentProviderName) -> Bool {
        providerHasStatus
    }

    public func getPaymentStatus(provider _: PaymentProviderName, paymentId: String) async throws -> PaymentOutcome {
        requestedPaymentIds.append(paymentId)
        return result
    }

    public func paymentIds() -> [String] {
        requestedPaymentIds
    }
}
