// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct NFTAssetId: Equatable, Hashable, Sendable {
    public let chain: Chain
    public let contractAddress: String
    public let tokenId: String

    public init(chain: Chain, contractAddress: String, tokenId: String) {
        self.chain = chain
        self.contractAddress = contractAddress
        self.tokenId = tokenId
    }
}

extension NFTAssetId: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let stringValue = try container.decode(String.self)
        self = try NFTAssetId(id: stringValue)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(identifier)
    }
}
