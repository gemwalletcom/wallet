// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemSearchScope
import protocol Gemstone.GemSearchServiceProtocol
import Primitives

public extension GemSearchServiceProtocol {
    @discardableResult
    func search(wallet: Primitives.Wallet, query: String, scope: WalletSearchTag, currency: String) async throws -> Bool {
        try await search(wallet: wallet.json(), query: query, scope: scope.gemScope, currency: Primitives.Currency(id: currency).json())
    }

    func searchAssets(wallet: Primitives.Wallet, query: String, currency: String) async throws -> [Primitives.AssetBasic] {
        try await searchAssets(wallet: wallet.json(), query: query, currency: Primitives.Currency(id: currency).json())
            .map { try Primitives.AssetBasic($0) }
    }
}

public extension WalletSearchTag {
    var gemScope: GemSearchScope {
        switch self {
        case .all: .all
        case let .list(id): .list(id: id)
        }
    }
}
