// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import ChainService
import ChainServiceTestKit
import Foundation
import class Gemstone.GemAssetsService
import protocol Gemstone.GemAssetsServiceProtocol
import class Gemstone.GemPriceService
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit

public extension AssetsService {
    static func mock(
        assetStore: AssetStore = .mock(),
        balanceStore: BalanceStore = .mock(),
        priceService: GemPriceService = .mock(),
        chainServiceFactory: any ChainServiceFactorable = ChainServiceFactoryMock(),
        assetsProvider: (any GemAssetsServiceProtocol)? = nil,
    ) -> AssetsService {
        AssetsService(
            assetStore: assetStore,
            balanceStore: balanceStore,
            chainServiceFactory: chainServiceFactory,
            assetsProvider: assetsProvider ?? GemAssetsService.mock(assetStore: assetStore, balanceStore: balanceStore, priceService: priceService),
        )
    }
}
