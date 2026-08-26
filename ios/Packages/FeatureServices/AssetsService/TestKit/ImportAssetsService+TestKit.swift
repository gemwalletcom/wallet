// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Foundation
import class Gemstone.GemAssetsService
import protocol Gemstone.GemAssetsServiceProtocol
import GemAPI
import GemAPITestKit
import Preferences
import PreferencesTestKit
import Store
import StoreTestKit

public extension ImportAssetsService {
    static func mock(
        assetListService: any GemAPIAssetsListService = GemAPIAssetsListServiceMock(),
        assetsProvider: any GemAssetsServiceProtocol = GemAssetsService.mock(),
        assetsService: AssetsService = .mock(),
        assetStore: AssetStore = .mock(),
        preferences: Preferences = .mock(),
    ) -> ImportAssetsService {
        ImportAssetsService(
            assetListService: assetListService,
            assetsProvider: assetsProvider,
            assetsService: assetsService,
            assetStore: assetStore,
            preferences: preferences,
        )
    }
}
