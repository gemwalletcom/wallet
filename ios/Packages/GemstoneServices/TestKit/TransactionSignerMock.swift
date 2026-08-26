// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemSignedTransaction
public import GemstonePrimitives
import GemstoneServices
import Primitives

public struct TransactionSignerMock: TransactionSigning {
    public static let transferType = (try? Primitives.TransactionType.transfer.json()) ?? ""

    public let signedTransactions: [GemSignedTransaction]

    public init(signedTransactions: [GemSignedTransaction] = [GemSignedTransaction(data: "signed_data", transactionType: TransactionSignerMock.transferType)]) {
        self.signedTransactions = signedTransactions
    }

    public func sign(
        transfer _: TransferData,
        transactionData _: TransactionData,
        amount _: TransferAmount,
        wallet _: Wallet,
    ) throws -> [GemSignedTransaction] {
        signedTransactions
    }
}

public extension GemSignedTransaction {
    init(data: String, type: Primitives.TransactionType) {
        self.init(data: data, transactionType: (try? type.json()) ?? "")
    }
}
