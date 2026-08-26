// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import Foundation
import protocol Gemstone.GemBannerServiceProtocol
import GemstonePrimitivesTestKit

public extension BannerSetupRunner {
    static func mock(
        bannerService: any GemBannerServiceProtocol = GemBannerServiceMock(),
    ) -> BannerSetupRunner {
        BannerSetupRunner(bannerService: bannerService)
    }
}
