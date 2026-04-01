// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemAssetBalance {
    func map() throws -> AssetBalance {
        try AssetBalance(
            assetId: AssetId(id: assetId),
            balance: balance.map(),
            isActive: isActive,
        )
    }
}
