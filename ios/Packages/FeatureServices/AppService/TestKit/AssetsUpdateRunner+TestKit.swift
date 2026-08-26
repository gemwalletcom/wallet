// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import GemstoneServices
import GemstoneServicesTestKit
import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import Primitives
import PrimitivesTestKit

public extension AssetsUpdateRunner {
    static func mock(
        configService: ConfigService = .mock(),
        assetsProvider: any GemAssetsServiceProtocol = GemAssetsServiceMock(),
        assetsService: AssetsService = .mock(),
        swappableChainsProvider: any SwappableChainsProvider = SwappableChainsProviderMock.mock(),
    ) -> AssetsUpdateRunner {
        AssetsUpdateRunner(
            configService: configService,
            assetsProvider: assetsProvider,
            assetsService: assetsService,
            swappableChainsProvider: swappableChainsProvider,
        )
    }
}
