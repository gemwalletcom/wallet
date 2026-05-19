// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct NFTCollectionId: Equatable, Hashable, Sendable {
    public let chain: Chain
    public let contractAddress: String

    public init(chain: Chain, contractAddress: String) {
        self.chain = chain
        self.contractAddress = contractAddress
    }

    public var identifier: String {
        "\(chain.rawValue)_\(contractAddress)"
    }

    public static func from(id: String) throws -> NFTCollectionId {
        guard let (chain, contractAddress) = AssetId.getData(id: id), let contractAddress else {
            throw AnyError("invalid nft collection id: \(id)")
        }
        return NFTCollectionId(chain: chain, contractAddress: contractAddress)
    }
}

extension NFTCollectionId: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        self = try Self.from(id: container.decode(String.self))
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(identifier)
    }
}
