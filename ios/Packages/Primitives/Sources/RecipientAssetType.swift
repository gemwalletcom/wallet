// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum RecipientAssetType: Codable, Equatable, Hashable, Sendable {
    case asset(Asset)
    case nft(NFTAsset)
}

extension RecipientAssetType: Identifiable {
    public var id: String {
        switch self {
        case let .asset(asset): asset.id.identifier
        case let .nft(asset): asset.id.identifier
        }
    }
}
