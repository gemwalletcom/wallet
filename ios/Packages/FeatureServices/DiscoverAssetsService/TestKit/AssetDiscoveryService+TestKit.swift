// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import AssetsServiceTestKit
import BalanceService
import BalanceServiceTestKit
import DiscoverAssetsService
import protocol Gemstone.GemTransactionsServiceProtocol
import GemstonePrimitivesTestKit
import NFTService
import NFTServiceTestKit
import TransactionsService
import TransactionsServiceTestKit

public extension AssetDiscoverable where Self == AssetDiscoveryService {
    static func mock(
        assetsListService: any GemTransactionsServiceProtocol = GemTransactionsServiceMock(),
        assetService: AssetsService = .mock(),
        assetsEnabler: any AssetsEnabler = .mock(),
        transactionsService: TransactionsService = .mock(),
        nftService: NFTService = .mock(),
    ) -> AssetDiscoveryService {
        AssetDiscoveryService(
            assetsListService: assetsListService,
            assetService: assetService,
            assetsEnabler: assetsEnabler,
            transactionsService: transactionsService,
            nftService: nftService,
        )
    }
}
