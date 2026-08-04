// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives

public struct PaymentAssetsProvidableMock: PaymentAssetsProvidable {
    private let assetsData: [AssetData]

    public init(assetsData: [AssetData] = []) {
        self.assetsData = assetsData
    }

    public func assetsData(walletId _: WalletId, assetIds: [AssetId]) -> [AssetData] {
        assetsData.filter { assetIds.contains($0.asset.id) }
    }
}
