// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public extension TransferData {
    static func mock(
        type: TransferDataType = .transfer(.mock()),
        recipient: RecipientData = .mock(),
        value: BigInt = .zero,
        canChangeValue: Bool = true,
    ) -> TransferData {
        TransferData(
            type: type,
            recipientData: recipient,
            value: value,
            canChangeValue: canChangeValue,
        )
    }
}
