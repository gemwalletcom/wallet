// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Primitives.AssetId {
    var assetType: AssetType? {
        guard let type = ChainConfig.config(chain: chain).defaultAssetType else {
            return .none
        }
        return AssetType(rawValue: type)
    }

    func getAssetType() throws -> AssetType {
        guard let assetType else {
            throw AnyError("Unknown asset type")
        }
        return assetType
    }
}
