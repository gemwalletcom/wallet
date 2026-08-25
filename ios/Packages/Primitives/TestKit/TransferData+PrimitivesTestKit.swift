// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public extension TransferData {
    static func mock(
        type: TransferDataType = .transfer(.mock()),
        recipient: RecipientData = .mock(),
        amount: TransferAmountValue = .exact(.zero),
        minimumValue: BigInt? = nil,
    ) -> TransferData {
        TransferData(
            type: type,
            recipientData: recipient,
            amount: amount,
            minimumValue: minimumValue,
        )
    }

    static func mockPayment(
        asset: Asset = .mockSolana(),
        transaction: String = "transaction",
        recipient: RecipientData = .mock(),
        amount: TransferAmountValue = .exact(.zero),
        transactionType: TransactionType = .transfer,
    ) -> TransferData {
        .mock(
            type: .generic(
                asset: asset,
                metadata: .mock(source: .payment),
                extra: .mock(
                    data: Data(transaction.utf8),
                    outputType: .encodedTransaction,
                    outputAction: .send,
                    transactionType: transactionType,
                ),
            ),
            recipient: recipient,
            amount: amount,
        )
    }
}
