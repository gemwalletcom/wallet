// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension TransactionSimulationServiceProtocol {
    func simulateTransaction(
        chain: Primitives.Chain,
        transaction: String,
        signerAddress: String,
    ) async throws -> Primitives.SimulationResult {
        try await simulateTransaction(
            chain: chain.map(),
            encodedTransaction: transaction,
            signerAddress: signerAddress,
        ).map()
    }
}
