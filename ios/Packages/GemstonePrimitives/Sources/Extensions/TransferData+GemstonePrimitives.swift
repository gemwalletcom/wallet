// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.TransactionInputType
import struct Gemstone.GemTransferData
import struct Gemstone.PaymentInvoice
import Primitives

public extension GemTransferData {
    var asset: Primitives.Asset {
        inputAsset().toPrimitives()
    }

    var chain: Chain {
        asset.chain
    }

    var invoice: PaymentInvoice? {
        switch inputType {
        case let .payment(_, invoice, _): invoice
        default: nil
        }
    }

    var applicationMetadata: Primitives.ApplicationMetadata? {
        guard case let .generic(_, metadata, _) = inputType else { return nil }
        return metadata.toPrimitives()
    }

    var id: String {
        identifier()
    }
}
