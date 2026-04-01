// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public typealias TransferDataAction = ((TransferData) -> Void)?

public extension TransferData {
    func updateValue(_ newValue: BigInt) -> TransferData {
        TransferData(
            type: type,
            recipientData: recipientData,
            value: newValue,
        )
    }
}
