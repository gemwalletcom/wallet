// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemPerpetualData {
    func map() throws -> PerpetualData {
        try PerpetualData(
            perpetual: perpetual.map(),
            asset: asset.map(),
            metadata: metadata.map(),
        )
    }
}
