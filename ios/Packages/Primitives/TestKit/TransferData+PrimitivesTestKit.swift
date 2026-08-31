// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public extension TransferData {
    static func mock(
        type: TransferDataType = .transfer(.mock()),
        recipient: Recipient = .mock(),
        value: BigInt = .zero,
        useMaxAmount: Bool = false,
        minimumValue: BigInt? = nil,
    ) -> TransferData {
        TransferData(
            type: type,
            recipient: recipient,
            value: value,
            useMaxAmount: useMaxAmount,
            minimumValue: minimumValue,
        )
    }

    static func mockPayment(
        asset: Asset = .mockSolana(),
        transaction: String = "transaction",
        recipient: Recipient = .mock(),
        value: BigInt = .zero,
    ) -> TransferData {
        .mock(
            type: .generic(
                asset: asset,
                metadata: .mock(source: .payment),
                extra: .mock(
                    data: Data(transaction.utf8),
                    outputType: .encodedTransaction,
                    outputAction: .send,
                    transactionType: .transfer,
                ),
            ),
            recipient: recipient,
            value: value,
        )
    }
}
