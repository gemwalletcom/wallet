// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemRecipient
import enum Gemstone.GemTransactionInputType
import BigInt
import Foundation
import GemstonePrimitives
import struct Gemstone.TransferDataExtra
import Primitives
import PrimitivesTestKit
import struct Gemstone.GemTransferData

public extension GemTransferData {
    static func mock(
        type: GemTransactionInputType = .transfer(.mock()),
        recipient: GemRecipient = .mock(),
        value: BigInt = .zero,
        useMaxAmount: Bool = false,
        minimumValue: BigInt? = nil,
    ) -> GemTransferData {
        GemTransferData(
            inputType: type,
            recipient: recipient,
            value: value,
            useMaxAmount: useMaxAmount,
            minimumValue: minimumValue,
        )
    }

    static func mockPayment(
        asset: Asset = .mockSolana(),
        transaction: String = "transaction",
        recipient: GemRecipient = .mock(),
        value: BigInt = .zero,
    ) -> GemTransferData {
        .mock(
            type: .generic(
                asset: asset,
                metadata: .mock(source: .payment),
                extra: .mock(data: Data(transaction.utf8)),
            ),
            recipient: recipient,
            value: value,
        )
    }
}
