// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAddAssetServiceProtocol
import Primitives

public extension GemAddAssetServiceProtocol {
    func token(chain: Chain, address: String) async throws -> Asset {
        try await token(chain: chain.rawValue, address: address).toPrimitives()
    }

    func add(wallet: Wallet, assetId: AssetId) async throws {
        try await add(wallet: wallet.toGem(), assetId: assetId.identifier)
    }
}
