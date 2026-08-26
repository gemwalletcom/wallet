// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitives
import Primitives

public final class GemAssetsServiceMock: GemAssetsServiceProtocol, @unchecked Sendable {
    private let searchAssetsResult: [Primitives.AssetBasic]
    private let assetsResult: [Primitives.AssetBasic]
    private let assetResult: Primitives.AssetFull?
    private let buyableFiatAssets: Primitives.FiatAssets?
    private let sellableFiatAssets: Primitives.FiatAssets?
    private let swapAssets: Primitives.FiatAssets?
    private let store: (any GemAssetStore)?

    public init(
        searchAssetsResult: [Primitives.AssetBasic] = [],
        assetsResult: [Primitives.AssetBasic] = [],
        assetResult: Primitives.AssetFull? = nil,
        buyableFiatAssets: Primitives.FiatAssets? = nil,
        sellableFiatAssets: Primitives.FiatAssets? = nil,
        swapAssets: Primitives.FiatAssets? = nil,
        store: (any GemAssetStore)? = nil,
    ) {
        self.searchAssetsResult = searchAssetsResult
        self.assetsResult = assetsResult
        self.assetResult = assetResult
        self.buyableFiatAssets = buyableFiatAssets
        self.sellableFiatAssets = sellableFiatAssets
        self.swapAssets = swapAssets
        self.store = store
    }

    public func getAsset(assetId _: Gemstone.AssetId) async throws -> Gemstone.AssetFull {
        guard let assetResult else { throw AnyError("not stubbed") }
        return try assetResult.json()
    }

    public func getAssets(assetIds _: [Gemstone.AssetId], currency _: String?) async throws -> [Gemstone.AssetBasic] {
        try assetsResult.map { try $0.json() }
    }

    public func getFiatAssets(quoteType: Gemstone.FiatQuoteType) async throws -> Gemstone.FiatAssets {
        let assets = try Primitives.FiatQuoteType(quoteType) == .buy ? buyableFiatAssets : sellableFiatAssets
        guard let assets else { throw AnyError("not stubbed") }
        return try assets.json()
    }

    public func getSwapAssets() async throws -> Gemstone.FiatAssets {
        guard let swapAssets else { throw AnyError("not stubbed") }
        return try swapAssets.json()
    }

    public func search(query _: String, chains _: [Gemstone.Chain], tags _: [String]) async throws -> Gemstone.SearchResponse {
        try Primitives.SearchResponse(assets: [], perpetuals: [], nfts: [], lists: []).json()
    }

    public func searchAssets(query _: String, chains _: [Gemstone.Chain]) async throws -> [Gemstone.AssetBasic] {
        try searchAssetsResult.map { try $0.json() }
    }

    public func prefetchAssets(assetIds: [Gemstone.AssetId]) async throws -> [Gemstone.AssetId] {
        guard let store else { return [] }
        let existing = try await store.getAssetIds(assetIds: assetIds).asSet()
        let missing = assetsResult.filter { assetIds.contains($0.asset.id.identifier) && !existing.contains($0.asset.id.identifier) }
        try await store.saveAssets(assets: missing.map { try $0.json() })
        return missing.map(\.asset.id.identifier)
    }

    public func getOrFetchAsset(assetId: Gemstone.AssetId) async throws -> Gemstone.Asset {
        guard let asset = assetsResult.first(where: { $0.asset.id.identifier == assetId }) else { throw AnyError("not stubbed") }
        return try asset.asset.json()
    }

    public func getOrFetchTokenAsset(assetId: Gemstone.AssetId) async throws -> Gemstone.Asset {
        try await getOrFetchAsset(assetId: assetId)
    }

    public func searchAssetsAndTokens(query _: String, chains _: [Gemstone.Chain]) async throws -> [Gemstone.AssetBasic] {
        try searchAssetsResult.map { try $0.json() }
    }

    public func searchTokens(tokenId _: String, chains _: [Gemstone.Chain]) async -> [Gemstone.AssetBasic] {
        []
    }

    public func addMissingBalances(walletId _: String, assetIds _: [Gemstone.AssetId]) async throws {}

    public func setupWallet(wallet _: Gemstone.Wallet) async throws {}

    public func setSwappableChains(chains _: [Gemstone.Chain]) async throws {}

    public func syncAvailability(versions _: Gemstone.ConfigVersions) async throws {}

    public func syncAsset(assetId _: Gemstone.AssetId, currency _: Gemstone.Currency) async throws -> Gemstone.AssetFull {
        guard let assetResult else { throw AnyError("not stubbed") }
        return try assetResult.json()
    }
}
