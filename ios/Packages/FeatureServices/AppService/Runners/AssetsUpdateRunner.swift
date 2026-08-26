// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Primitives
import Store

public struct AssetsUpdateRunner: AsyncRunnable {
    public let id = "assets_update"

    private let configService: ConfigService
    private let assetsProvider: any GemAssetsServiceProtocol
    private let assetStore: AssetStore
    private let swappableChainsProvider: any SwappableChainsProvider

    public init(
        configService: ConfigService,
        assetsProvider: any GemAssetsServiceProtocol,
        assetStore: AssetStore,
        swappableChainsProvider: any SwappableChainsProvider,
    ) {
        self.configService = configService
        self.assetsProvider = assetsProvider
        self.assetStore = assetStore
        self.swappableChainsProvider = swappableChainsProvider
    }

    public func run() async throws {
        try assetStore.setAssetIsSwappable(for: swappableChainsProvider.supportedChains().map(\.id), value: true)
        guard let config = await configService.getConfig() else {
            throw AnyError("Config not found")
        }
        try await assetsProvider.syncAvailability(versions: config.versions.json())
    }
}
