// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import Primitives

public extension GemAssetsServiceProtocol {
    func ensureAsset(for assetId: Primitives.AssetId) async throws -> Primitives.Asset {
        try await ensureAsset(assetId: assetId.identifier).toPrimitives()
    }
}
