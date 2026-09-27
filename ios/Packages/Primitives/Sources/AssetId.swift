// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct AssetId: Equatable, Hashable, Sendable {
    public let chain: Chain
    public let tokenId: String?

    public init(chain: Chain, tokenId: String?) {
        self.chain = chain
        self.tokenId = tokenId
    }
}

extension AssetId: IdentifierCodable {
    enum CodingKeys: String, CodingKey {
        case chain
        case tokenId
    }

    public init(from decoder: Decoder) throws {
        if let container = try? decoder.singleValueContainer(), let stringValue = try? container.decode(String.self) {
            self = try AssetId.from(id: stringValue)
            return
        }
        let container = try decoder.container(keyedBy: CodingKeys.self)
        chain = try container.decode(Chain.self, forKey: .chain)
        tokenId = try container.decodeIfPresent(String.self, forKey: .tokenId)
    }
}
