// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import Primitives

public extension GemAssetsServiceProtocol {
    func getOrFetchAsset(for assetId: Primitives.AssetId) async throws -> Primitives.Asset {
        try await Primitives.Asset(getOrFetchAsset(assetId: assetId.identifier))
    }

    func getOrFetchTokenAsset(for assetId: Primitives.AssetId) async throws -> Primitives.Asset {
        try await Primitives.Asset(getOrFetchTokenAsset(assetId: assetId.identifier))
    }

    @discardableResult
    func prefetchAssets(for assetIds: [Primitives.AssetId]) async throws -> [Primitives.AssetId] {
        try await prefetchAssets(assetIds: assetIds.ids).map { try Primitives.AssetId(id: $0) }
    }

    func addMissingBalances(walletId: Primitives.WalletId, assetIds: [Primitives.AssetId]) async throws {
        try await addMissingBalances(walletId: walletId.id, assetIds: assetIds.ids)
    }

    @discardableResult
    func syncAsset(for assetId: Primitives.AssetId, currency: String) async throws -> Primitives.AssetFull {
        guard let currency = Primitives.Currency(rawValue: currency) else {
            throw AnyError("unknown currency: \(currency)")
        }
        return try await Primitives.AssetFull(syncAsset(assetId: assetId.identifier, currency: currency.json()))
    }

    func searchAssetsAndTokens(query: String, chains: [Primitives.Chain]) async throws -> [Primitives.AssetBasic] {
        try await searchAssetsAndTokens(query: query, chains: chains.map(\.rawValue)).map { try Primitives.AssetBasic($0) }
    }

    func searchTokens(tokenId: String, chains: [Primitives.Chain]) async -> [Primitives.AssetBasic] {
        await searchTokens(tokenId: tokenId, chains: chains.map(\.rawValue)).compactMap { try? Primitives.AssetBasic($0) }
    }
}
