// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemTransactionInputType
import struct Gemstone.GemTransferData
import Primitives

public extension GemTransferData {
    var chain: Chain {
        inputType.chain
    }

    var id: String {
        [chain.rawValue, recipient.address, value.description].joined(separator: "-")
    }
}

