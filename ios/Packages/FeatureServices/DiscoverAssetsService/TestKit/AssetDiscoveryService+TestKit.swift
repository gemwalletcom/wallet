// Copyright (c). Gem Wallet. All rights reserved.

import DiscoverAssetsService
import protocol Gemstone.GemAssetDiscoveryServiceProtocol
import GemstonePrimitivesTestKit
import NFTService
import NFTServiceTestKit
import Preferences
import PreferencesTestKit
import TransactionsService
import TransactionsServiceTestKit

public extension AssetDiscoverable where Self == AssetDiscoveryService {
    static func mock(
        discovery: any GemAssetDiscoveryServiceProtocol = GemAssetDiscoveryServiceMock(),
        transactionsService: TransactionsService = .mock(),
        nftService: NFTService = .mock(),
        preferences: Preferences = .mock(),
    ) -> AssetDiscoveryService {
        AssetDiscoveryService(
            discovery: discovery,
            transactionsService: transactionsService,
            nftService: nftService,
            preferences: preferences,
        )
    }
}
