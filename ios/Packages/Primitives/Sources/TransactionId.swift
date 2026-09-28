// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct TransactionId: Equatable, Hashable, Sendable {
    public let chain: Chain
    public let hash: String

    public init(chain: Chain, hash: String) {
        self.chain = chain
        self.hash = hash
    }

    public static func from(id: String) throws -> TransactionId {
        guard let (chain, hash) = AssetId.getData(id: id), let hash else {
            throw AnyError("invalid transaction id: \(id)")
        }
        return TransactionId(chain: chain, hash: hash)
    }

    public var identifier: String {
        chain.rawValue + "_" + hash
    }
}

extension TransactionId: IdentifierCodable {}
