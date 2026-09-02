// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.GemTransactionInputType
import struct Gemstone.GemRecipient
import struct Gemstone.GemTransferData
import Primitives

public extension GemTransferData {
    init(
        inputType: GemTransactionInputType,
        recipient: GemRecipient,
        value: BigInt,
        useMaxAmount: Bool = false,
        minimumValue: BigInt? = nil,
    ) {
        self.init(
            inputType: inputType,
            recipient: recipient,
            value: value.description,
            useMaxAmount: useMaxAmount,
            minimumValue: minimumValue?.description,
        )
    }

    var chain: Chain {
        inputType.chain
    }

    var id: String {
        [chain.rawValue, recipient.address, value].joined(separator: "-")
    }
}

