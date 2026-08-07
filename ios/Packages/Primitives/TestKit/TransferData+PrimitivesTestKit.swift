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
}
