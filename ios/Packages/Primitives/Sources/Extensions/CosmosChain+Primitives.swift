// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension CosmosChain {
    static func from(string: String) throws -> CosmosChain {
        guard let chain = CosmosChain(rawValue: string) else {
            throw AnyError("Unknown cosmos chain: \(string)")
        }
        return chain
    }
}
