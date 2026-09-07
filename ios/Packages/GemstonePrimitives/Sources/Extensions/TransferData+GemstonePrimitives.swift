// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.TransactionInputType
import struct Gemstone.GemTransferData
import Primitives

public extension GemTransferData {
    var asset: Primitives.Asset {
        inputAsset().map()
    }

    var chain: Chain {
        asset.chain
    }

    var applicationMetadata: Primitives.ApplicationMetadata? {
        guard case let .generic(_, metadata, _) = inputType else { return nil }
        return metadata.map()
    }

    var id: String {
        identifier()
    }
}
