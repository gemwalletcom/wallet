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

    public init(
        searchAssetsResult: [Primitives.AssetBasic] = [],
        assetsResult: [Primitives.AssetBasic] = [],
        assetResult: Primitives.AssetFull? = nil,
        buyableFiatAssets: Primitives.FiatAssets? = nil,
        sellableFiatAssets: Primitives.FiatAssets? = nil,
        swapAssets: Primitives.FiatAssets? = nil,
    ) {
        self.searchAssetsResult = searchAssetsResult
        self.assetsResult = assetsResult
        self.assetResult = assetResult
        self.buyableFiatAssets = buyableFiatAssets
        self.sellableFiatAssets = sellableFiatAssets
        self.swapAssets = swapAssets
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
}
