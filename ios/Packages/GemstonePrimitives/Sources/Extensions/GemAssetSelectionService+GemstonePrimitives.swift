// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemSearchScope
import protocol Gemstone.GemAssetSelectionServiceProtocol
import Primitives

public extension GemAssetSelectionServiceProtocol {
    @discardableResult
    func search(query: String, scope: WalletSearchTag) async throws -> Bool {
        try await search(query: query, scope: scope.gemScope)
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
