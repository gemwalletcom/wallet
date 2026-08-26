// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstoneServices
import Foundation
import GemstonePrimitives
import Testing

struct ChainCoreErrorTests {
    private struct StubError: LocalizedError {
        let message: String
        var errorDescription: String? { message }
    }

    @Test
    func fromErrorMapsSignerDustError() {
        let error = GemstoneError.SignerError(error: .dustThreshold, msg: "message can change")
        #expect(ChainCoreError.fromError(error) == .dustThreshold)
    }

    @Test
    func fromErrorMapsSignerInsufficientFundsError() {
        let error = GemstoneError.SignerError(error: .insufficientFunds, msg: "message can change")
        #expect(ChainCoreError.fromError(error) == .insufficientBalance)
    }

    @Test
    func fromErrorDoesNotParseSignerMessages() {
        let error = StubError(message: "transaction amount is below the dust threshold")
        #expect(ChainCoreError.fromError(error) == nil)
    }
}
