// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import PrimitivesTestKit
import Preferences
import PreferencesTestKit
import Store
import StoreTestKit

public extension ImportAssetsService {
    static func mock(
        assetStore: AssetStore = .mock(),
        preferences: Preferences = .mock(),
    ) -> ImportAssetsService {
        ImportAssetsService(
            assetStore: assetStore,
            preferences: preferences,
        )
    }
}
