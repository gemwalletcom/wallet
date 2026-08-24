// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemSignedTransaction
import Primitives
import Signer

public struct TransactionSignerMock: TransactionSigning {
    public let signedTransactions: [GemSignedTransaction]

    public init(signedTransactions: [GemSignedTransaction] = [GemSignedTransaction(data: "signed_data", transactionType: .transfer)]) {
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
