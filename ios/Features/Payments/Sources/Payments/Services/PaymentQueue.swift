// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

actor PaymentQueue {
    private var last: Task<Void, Never>?
    private var payments: [String: Task<PaymentOutcome, any Error>] = [:]

    func enqueue(paymentId: String, work: @Sendable @escaping () async throws -> PaymentOutcome) -> Task<PaymentOutcome, any Error> {
        if let payment = payments[paymentId] {
            return payment
        }
        let previous = last
        let payment = Task {
            await previous?.value
            return try await work()
        }
        payments[paymentId] = payment
        last = Task { [weak self] in
            _ = try? await payment.value
            await self?.finish(paymentId: paymentId)
        }
        return payment
    }

    private func finish(paymentId: String) {
        payments[paymentId] = .none
    }
}
