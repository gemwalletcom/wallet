// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import GemstoneServices
import GemstoneServicesTestKit
import Foundation

public extension BannerSetupRunner {
    static func mock(
        bannerSetupService: BannerSetupService = .mock(),
    ) -> BannerSetupRunner {
        BannerSetupRunner(bannerSetupService: bannerSetupService)
    }
}
