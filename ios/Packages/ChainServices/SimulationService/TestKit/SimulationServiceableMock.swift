// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.Chain
import enum Gemstone.SignableTransactionType
import enum Gemstone.SignDigestType
import Primitives
import SimulationService

public struct SimulationServiceableMock: SimulationServiceable {
    private let result: SimulationResult

    public init(result: SimulationResult = .empty) {
        self.result = result
    }

    public func simulateSignMessage(chain _: Gemstone.Chain, signType _: SignDigestType, data _: String, sessionDomain _: String) async throws -> SimulationResult {
        result
    }

    public func simulateSendTransaction(chain _: Gemstone.Chain, transactionType _: SignableTransactionType, data _: String) async throws -> SimulationResult {
        result
    }
}
