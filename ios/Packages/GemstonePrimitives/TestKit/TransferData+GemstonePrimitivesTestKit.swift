// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemRecipient
import struct Gemstone.GemTransferData
import struct Gemstone.PaymentInvoice
import enum Gemstone.TransactionInputType
import struct Gemstone.TransferDataExtra
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

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
        asset: Asset = .mock(id: .mock(chain: .solana), name: "Solana", symbol: "SOL", decimals: 9),
        transaction: String = "transaction",
        recipient: GemRecipient = .mock(),
        value: BigInt = .zero,
        invoice: PaymentInvoice = .mock(),
    ) -> GemTransferData {
        .mock(
            type: .payment(
                asset: asset.toGem(),
                invoice: invoice,
                extra: .mock(data: Data(transaction.utf8)),
            ),
            recipient: recipient,
            value: value,
        )
    }
}
