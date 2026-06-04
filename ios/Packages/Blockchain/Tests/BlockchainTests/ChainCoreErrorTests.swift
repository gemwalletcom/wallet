// Copyright (c). Gem Wallet. All rights reserved.

@testable import Blockchain
import Foundation
import Testing

struct ChainCoreErrorTests {
    private struct StubError: LocalizedError {
        let message: String
        var errorDescription: String? { message }
    }

    @Test
    func fromErrorMapsNativeSignerDustMessage() {
        let error = StubError(message: "transaction amount is below the dust threshold")
        #expect(ChainCoreError.fromError(error) == .dustThreshold)
    }

    @Test
    func fromErrorMapsNativeSignerInsufficientBalanceMessage() {
        let error = StubError(message: "insufficient balance")
        #expect(ChainCoreError.fromError(error) == .insufficientBalance)
    }

    @Test
    func fromErrorReturnsNilForUnrelatedMessage() {
        let error = StubError(message: "broadcast failed: node offline")
        #expect(ChainCoreError.fromError(error) == nil)
    }
}
