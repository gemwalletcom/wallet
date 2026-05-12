// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct NFTCollectionId: Equatable, Hashable, Sendable {
    public let chain: Chain
    public let contractAddress: String

    public init(chain: Chain, contractAddress: String) {
        self.chain = chain
        self.contractAddress = contractAddress
    }
}

extension NFTCollectionId: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let stringValue = try container.decode(String.self)
        self = try NFTCollectionId(id: stringValue)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(identifier)
    }
}
