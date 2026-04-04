// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import AssetsServiceTestKit
import BalanceService
import BalanceServiceTestKit
import DiscoverAssetsService
import GemAPI
import GemAPITestKit
import NFTService
import NFTServiceTestKit
import TransactionsService
import TransactionsServiceTestKit

public extension AssetDiscoverable where Self == AssetDiscoveryService {
    static func mock(
        assetsListService: any GemAPIAssetsListService = GemAPIAssetsListServiceMock(assetsByDeviceIdResult: []),
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
