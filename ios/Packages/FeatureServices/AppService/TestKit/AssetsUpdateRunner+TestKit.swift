// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import AssetsService
import AssetsServiceTestKit
import Foundation
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit

public extension AssetsUpdateRunner {
    static func mock(
        configService: ConfigService = .mock(),
        importAssetsService: ImportAssetsService = .mock(),
        assetsService: AssetsService = .mock(),
        swappableChainsProvider: any SwappableChainsProvider = SwappableChainsProviderMock.mock(),
        preferences: Preferences = .mock(),
    ) -> AssetsUpdateRunner {
        AssetsUpdateRunner(
            configService: configService,
            importAssetsService: importAssetsService,
            assetsService: assetsService,
            swappableChainsProvider: swappableChainsProvider,
            preferences: preferences,
        )
    }
}
