// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import GemstonePrimitives
import Primitives

public struct AssetsUpdateRunner: AsyncRunnable {
    public let id = "assets_update"

    private let configService: ConfigService
    private let assetsProvider: any GemAssetsServiceProtocol
    private let assetsService: AssetsService
    private let swappableChainsProvider: any SwappableChainsProvider

    public init(
        configService: ConfigService,
        assetsProvider: any GemAssetsServiceProtocol,
        assetsService: AssetsService,
        swappableChainsProvider: any SwappableChainsProvider,
    ) {
        self.configService = configService
        self.assetsProvider = assetsProvider
        self.assetsService = assetsService
        self.swappableChainsProvider = swappableChainsProvider
    }

    public func run() async throws {
        try assetsService.setSwappableAssets(for: swappableChainsProvider.supportedChains())
        guard let config = await configService.getConfig() else {
            throw AnyError("Config not found")
        }
        try await assetsProvider.syncAvailability(versions: config.versions.json())
    }
}
