// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public struct TransferData: Identifiable, Sendable, Hashable {
    public let type: TransferDataType
    public let recipientData: RecipientData
    public let value: BigInt
    public let minimumValue: BigInt?
    public let canChangeValue: Bool

    public init(
        type: TransferDataType,
        recipientData: RecipientData,
        value: BigInt,
        minimumValue: BigInt? = nil,
        canChangeValue: Bool = true,
    ) {
        self.type = type
        self.recipientData = recipientData
        self.value = value
        self.minimumValue = minimumValue
        self.canChangeValue = canChangeValue
    }

    public var id: String {
        [type.transactionType.rawValue, recipientData.recipient.address, String(value)].joined(separator: "-")
    }

    public var chain: Chain {
        type.chain
    }
}
