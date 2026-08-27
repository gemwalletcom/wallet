// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import Primitives

public extension GemAssetsServiceProtocol {
    func ensureAsset(for assetId: Primitives.AssetId) async throws -> Primitives.Asset {
        try await Primitives.Asset(ensureAsset(assetId: assetId.identifier))
    }

    func openWalletAsset(wallet: Primitives.Wallet, assetId: Primitives.AssetId) async throws -> Primitives.Asset? {
        try await openWalletAsset(wallet: wallet.json(), assetId: assetId.identifier).map { try Primitives.Asset($0) }
    }

    func ensureTokenAsset(for assetId: Primitives.AssetId) async throws -> Primitives.Asset {
        try await Primitives.Asset(ensureTokenAsset(assetId: assetId.identifier))
    }

    @discardableResult
    func prefetchAssets(for assetIds: [Primitives.AssetId]) async throws -> [Primitives.AssetId] {
        try await prefetchAssets(assetIds: assetIds.ids).map { try Primitives.AssetId(id: $0) }
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
