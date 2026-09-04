// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAddAssetServiceProtocol
import Primitives

public extension GemAddAssetServiceProtocol {
    func chains(wallet: Wallet) -> [Chain] {
        chains(accounts: wallet.accounts.map { $0.map() }).map { Chain(core: $0) }
    }

    func defaultChain(chains: [Chain]) -> Chain? {
        defaultChain(chains: chains.map(\.rawValue)).map { Chain(core: $0) }
    }

    func tokenUrl(chain: Chain, tokenId: String) -> BlockExplorerLink? {
        tokenUrl(chain: chain.rawValue, tokenId: tokenId).map { BlockExplorerLink($0) }
    }

    func token(chain: Chain, address: String) async throws -> Asset {
        try await token(chain: chain.rawValue, address: address).map()
    }

    func add(wallet: Wallet, assetId: AssetId) async throws {
        try await add(wallet: wallet.json(), assetId: assetId.identifier)
    }
}
