// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemSignedTransaction
import GemstonePrimitives
import Primitives

public protocol TransactionSigning: Sendable {
    func sign(
        transfer: TransferData,
        transactionData: TransactionData,
        amount: TransferAmount,
        wallet: Wallet,
    ) async throws -> [GemSignedTransaction]
}
