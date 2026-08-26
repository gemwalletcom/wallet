// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public actor GemAPIAssetsServiceMock: GemAPIAssetsService {
    private var assetsResult: [AssetBasic]?

    public init(assetsResult: [AssetBasic]? = nil) {
        self.assetsResult = assetsResult
    }

    public func getAssets(currency _: String?, assetIds _: [AssetId]) async throws -> [AssetBasic] {
        assetsResult ?? []
    }
}
