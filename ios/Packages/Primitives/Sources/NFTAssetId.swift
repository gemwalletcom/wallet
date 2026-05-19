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

    public var identifier: String {
        "\(chain.rawValue)_\(contractAddress)\(AssetId.subTokenSeparator)\(tokenId)"
    }

    public static func from(id: String) throws -> NFTAssetId {
        guard let (chain, rest) = AssetId.getData(id: id), let rest else {
            throw AnyError("invalid nft asset id: \(id)")
        }
        let parts = rest.split(separator: AssetId.subTokenSeparator, maxSplits: 1, omittingEmptySubsequences: false)
        guard parts.count == 2 else {
            throw AnyError("invalid nft asset id: \(id)")
        }
        return NFTAssetId(chain: chain, contractAddress: String(parts[0]), tokenId: String(parts[1]))
    }
}

extension NFTAssetId: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        self = try Self.from(id: container.decode(String.self))
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(identifier)
    }
}
