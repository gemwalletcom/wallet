// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionInputType
import BigInt
import Foundation
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension TransferData {
    static func mock(
        type: GemTransactionInputType = .transfer(.mock()),
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
