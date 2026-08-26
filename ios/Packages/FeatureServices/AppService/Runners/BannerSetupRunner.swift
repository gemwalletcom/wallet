// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemBannerServiceProtocol
import Primitives

public struct BannerSetupRunner: AsyncRunnable {
    public let id = "banner_setup"

    private let bannerService: any GemBannerServiceProtocol

    public init(bannerService: any GemBannerServiceProtocol) {
        self.bannerService = bannerService
    }

    public func run() async throws {
        try await bannerService.setup()
    }
}
