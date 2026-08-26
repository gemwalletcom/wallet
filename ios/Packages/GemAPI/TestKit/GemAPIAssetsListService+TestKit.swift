// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public actor GemAPIAssetsListServiceMock: GemAPIAssetsListService {
    private var assetsByDeviceIdResult: [AssetId]

    public init(assetsByDeviceIdResult: [AssetId] = []) {
        self.assetsByDeviceIdResult = assetsByDeviceIdResult
    }

    public func getDeviceAssets(walletId _: WalletId, fromTimestamp _: Int) async throws -> [AssetId] {
        assetsByDeviceIdResult
    }
}
