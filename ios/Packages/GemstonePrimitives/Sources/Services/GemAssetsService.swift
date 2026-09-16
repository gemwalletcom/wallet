// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import Primitives

public extension GemAssetsServiceProtocol {
    func ensureAsset(for assetId: Primitives.AssetId) async throws -> Primitives.Asset {
        try await ensureAsset(assetId: assetId.identifier).toPrimitives()
    }

    func openWalletAsset(wallet: Primitives.Wallet, assetId: Primitives.AssetId) async throws -> Primitives.Asset? {
        try await openWalletAsset(wallet: wallet.toGem(), assetId: assetId.identifier).map { $0.toPrimitives() }
    }

    func openAsset(for assetId: Primitives.AssetId) async throws -> Primitives.Asset? {
        try await openAsset(assetId: assetId.identifier).map { $0.toPrimitives() }
    }

    func ensureTokenAsset(for assetId: Primitives.AssetId) async throws -> Primitives.Asset {
        try await ensureTokenAsset(assetId: assetId.identifier).toPrimitives()
    }
}
