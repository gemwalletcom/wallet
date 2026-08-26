// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import ChainService
import ChainServiceTestKit
import Foundation
import class Gemstone.GemAssetsService
import protocol Gemstone.GemAssetsServiceProtocol
import Primitives
import Store
import StoreTestKit

public extension AssetsService {
    static func mock(
        assetStore: AssetStore = .mock(),
        balanceStore: BalanceStore = .mock(),
        priceStore: PriceStore = .mock(),
        chainServiceFactory: any ChainServiceFactorable = ChainServiceFactoryMock(),
        assetsProvider: any GemAssetsServiceProtocol = GemAssetsService.mock(),
    ) -> AssetsService {
        AssetsService(
            assetStore: assetStore,
            balanceStore: balanceStore,
            priceStore: priceStore,
            chainServiceFactory: chainServiceFactory,
            assetsProvider: assetsProvider,
        )
    }
}
