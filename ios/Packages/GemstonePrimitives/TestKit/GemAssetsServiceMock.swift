// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public final class GemAssetStoreMock: GemAssetStore, @unchecked Sendable {
    public var assets: [Gemstone.Asset]

    public init(assets: [Gemstone.Asset] = []) {
        self.assets = assets
    }

    public func getAssetIds(assetIds: [Gemstone.AssetId]) async throws -> [Gemstone.AssetId] {
        assetIds
    }

    public func getAssets(assetIds: [Gemstone.AssetId]) async throws -> [Gemstone.Asset] {
        assets.filter { assetIds.contains($0.id) }
    }

    public func getWalletAssets(walletId _: Gemstone.WalletId, filters _: [Gemstone.GemAssetFilter]) async throws -> [Gemstone.Asset] {
        assets
    }

    public func getAssetBasics(assetIds _: [Gemstone.AssetId]) async throws -> [Gemstone.AssetBasic] {
        []
    }

    public func saveAssets(assets _: [Gemstone.AssetBasic]) async throws {}
    public func saveAsset(asset _: Gemstone.AssetFull) async throws {}
    public func setBuyableAssets(assetIds _: [Gemstone.AssetId]) async throws {}
    public func setSellableAssets(assetIds _: [Gemstone.AssetId]) async throws {}
    public func setSwappableAssets(assetIds _: [Gemstone.AssetId]) async throws {}
    public func setStakeableAssets(assetIds _: [Gemstone.AssetId]) async throws {}
}

final class GemPriceStoreMock: GemPriceStore, @unchecked Sendable {
    init() {}

    func getPrices(assetIds _: [Gemstone.AssetId]) async throws -> [Gemstone.AssetPrice] {
        []
    }

    func getRate(currency _: Gemstone.Currency) async throws -> Gemstone.FiatRate? {
        nil
    }

    func getRates() async throws -> [Gemstone.FiatRate] {
        []
    }

    func saveRatesAndPrices(currency _: Gemstone.Currency, rates _: [Gemstone.FiatRate], conversion _: Gemstone.FiatRate?, prices _: [Gemstone.GemPriceUpdate]) async throws {}
    func savePrices(currency _: Gemstone.Currency, prices _: [Gemstone.GemPriceUpdate]) async throws {}
    func convertPrices(currency _: Gemstone.Currency, rate _: Double) async throws {}
    func saveMarket(assetId _: Gemstone.AssetId, market _: Gemstone.AssetMarket) async throws {}
}

public extension GemAssetsService {
    static func mock(store: any GemAssetStore = GemAssetStoreMock()) -> GemAssetsService {
        let provider = StubAlienProvider()
        let preferences = GemPreferencesStoreMock()
        return GemAssetsService(
            api: GemApiClient(provider: provider),
            gateway: GemGateway(provider: provider, nodes: .mock(), preferences: preferences, securePreferences: GemSecureStoreMock()),
            store: store,
            price: GemPriceService(store: GemPriceStoreMock(), preferences: GemPreferencesService(store: preferences)),
            preferences: GemPreferencesService(store: preferences),
        )
    }
}
