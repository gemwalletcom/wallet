// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import ChainService
import ChainServiceTestKit
import Foundation
import class Gemstone.GemAssetsService
import protocol Gemstone.GemAssetsServiceProtocol
import PriceService
import PriceServiceTestKit
import Primitives
import Store
import StoreTestKit

public extension AssetsService {
    static func mock(
        assetStore: AssetStore = .mock(),
        balanceStore: BalanceStore = .mock(),
        priceService: PriceService = .mock(),
        chainServiceFactory: any ChainServiceFactorable = ChainServiceFactoryMock(),
        assetsProvider: (any GemAssetsServiceProtocol)? = nil,
    ) -> AssetsService {
        AssetsService(
            assetStore: assetStore,
            balanceStore: balanceStore,
            priceService: priceService,
            chainServiceFactory: chainServiceFactory,
            assetsProvider: assetsProvider ?? GemAssetsService.mock(assetStore: assetStore, balanceStore: balanceStore),
        )
    }
}
