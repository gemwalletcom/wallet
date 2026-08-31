// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public struct TransferData: Identifiable, Sendable, Hashable {
    public let type: TransferDataType
    public let recipient: Recipient
    public let value: BigInt
    public let useMaxAmount: Bool
    public let minimumValue: BigInt?

    public init(
        type: TransferDataType,
        recipient: Recipient,
        value: BigInt,
        useMaxAmount: Bool = false,
        minimumValue: BigInt? = nil,
    ) {
        self.type = type
        self.recipient = recipient
        self.value = value
        self.useMaxAmount = useMaxAmount
        self.minimumValue = minimumValue
    }

    public var id: String {
        [type.chain.rawValue, recipient.address, String(value)].joined(separator: "-")
    }

    public var chain: Chain {
        type.chain
    }
}
