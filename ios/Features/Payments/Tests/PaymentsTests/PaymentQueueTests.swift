// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing
@testable import Payments

struct PaymentQueueTests {
    @Test
    func joinsThePaymentAlreadyInFlight() async throws {
        let queue = PaymentQueue()
        let gate = Gate()

        let first = await queue.enqueue(paymentId: "pay_1") {
            await gate.wait()
            return .mock(status: .succeeded)
        }
        let second = await queue.enqueue(paymentId: "pay_1") {
            Issue.record("Expected the payment in flight to be reused")
            return .mock(status: .failed)
        }

        await gate.open()
        #expect(try await first.value.status == .succeeded)
        #expect(try await second.value.status == .succeeded)
    }

    @Test
    func runsPaymentsOneAfterAnother() async throws {
        let queue = PaymentQueue()
        let gate = Gate()
        let order = Order()

        let first = await queue.enqueue(paymentId: "pay_1") {
            await gate.wait()
            await order.append("first")
            return .mock()
        }
        let second = await queue.enqueue(paymentId: "pay_2") {
            await order.append("second")
            return .mock()
        }

        #expect(await order.values.isEmpty)
        await gate.open()
        _ = try await first.value
        _ = try await second.value
        #expect(await order.values == ["first", "second"])
    }
}

private actor Gate {
    private var continuation: CheckedContinuation<Void, Never>?
    private var isOpen = false

    func wait() async {
        guard !isOpen else { return }
        await withCheckedContinuation { continuation = $0 }
    }

    func open() {
        isOpen = true
        continuation?.resume()
        continuation = nil
    }
}

private actor Order {
    private(set) var values: [String] = []

    func append(_ value: String) {
        values.append(value)
    }
}
