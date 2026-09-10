// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemRecipient
import enum Gemstone.TransactionInputType
import BigInt
import Foundation
import GemstonePrimitives
import struct Gemstone.TransferDataExtra
import Primitives
import PrimitivesTestKit
import struct Gemstone.GemTransferData
import struct Gemstone.PaymentInvoice

public extension GemTransferData {
    static func mock(
        type: TransactionInputType = .transfer(.mock()),
        recipient: GemRecipient = .mock(),
        value: BigInt = .zero,
        useMaxAmount: Bool = false,
    ) -> GemTransferData {
        GemTransferData(
            inputType: type,
            recipient: recipient,
            value: value,
            useMaxAmount: useMaxAmount,
        )
    }

    static func mockPayment(
        asset: Asset = .mockSolana(),
        transaction: String = "transaction",
        recipient: GemRecipient = .mock(),
        value: BigInt = .zero,
        invoice: PaymentInvoice = .mock(),
    ) -> GemTransferData {
        .mock(
            type: .payment(
                asset: asset.map(),
                invoice: invoice,
                extra: .mock(data: Data(transaction.utf8)),
            ),
            recipient: recipient,
            value: value,
        )
    }
}
