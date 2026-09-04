// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Primitives.AssetId {
    var assetType: AssetType? {
        ChainConfig.config(chain: chain).defaultAssetType?.map()
    }
}
