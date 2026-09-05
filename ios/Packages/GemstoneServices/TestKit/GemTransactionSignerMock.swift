// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemSignedTransaction
public import struct Gemstone.GemSignerInput
public import protocol Gemstone.GemTransactionSigner
public import struct Gemstone.Wallet
public import GemstonePrimitives
import Primitives

public final class GemTransactionSignerMock: GemTransactionSigner {
    public static let transferType = Primitives.TransactionType.transfer.map()

    public let signedTransactions: [GemSignedTransaction]

    public init(signedTransactions: [GemSignedTransaction] = [GemSignedTransaction(data: "signed_data", transactionType: GemTransactionSignerMock.transferType)]) {
        self.signedTransactions = signedTransactions
    }

    public func sign(wallet _: Gemstone.Wallet, input _: GemSignerInput) throws -> [GemSignedTransaction] {
        signedTransactions
    }
}

public extension GemSignedTransaction {
    init(data: String, type: Primitives.TransactionType) {
        self.init(data: data, transactionType: type.map())
    }
}
