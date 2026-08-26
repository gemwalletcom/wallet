// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Foundation
import class Gemstone.GemAssetsService
import protocol Gemstone.GemAssetsServiceProtocol
import Preferences
import PreferencesTestKit
import Store
import StoreTestKit

public extension ImportAssetsService {
    static func mock(
        assetsProvider: any GemAssetsServiceProtocol = GemAssetsService.mock(),
        assetsService: AssetsService = .mock(),
        assetStore: AssetStore = .mock(),
        preferences: Preferences = .mock(),
    ) -> ImportAssetsService {
        ImportAssetsService(
            assetsProvider: assetsProvider,
            assetsService: assetsService,
            assetStore: assetStore,
            preferences: preferences,
        )
    }
}
