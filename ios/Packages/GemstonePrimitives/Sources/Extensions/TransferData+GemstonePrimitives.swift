// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemTransferData
import struct Gemstone.PaymentInvoice
import enum Gemstone.TransactionInputType
import Primitives

public extension GemTransferData {
    var asset: Primitives.Asset {
        inputAsset().toPrimitives()
    }

    var chain: Chain {
        asset.chain
    }

    var id: String {
        identifier()
    }
}
