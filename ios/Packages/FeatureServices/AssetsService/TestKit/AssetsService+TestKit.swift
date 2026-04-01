// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import ChainService
import ChainServiceTestKit
import Foundation
import GemAPI
import GemAPITestKit
import Primitives
import Store
import StoreTestKit

public extension AssetsService {
    static func mock(
        assetStore: AssetStore = .mock(),
        balanceStore: BalanceStore = .mock(),
        chainServiceFactory: any ChainServiceFactorable = ChainServiceFactoryMock(),
        assetsProvider: any GemAPIAssetsService = GemAPIAssetsServiceMock(),
    ) -> AssetsService {
        AssetsService(
            assetStore: assetStore,
            balanceStore: balanceStore,
            chainServiceFactory: chainServiceFactory,
            assetsProvider: assetsProvider,
        )
    }
}
