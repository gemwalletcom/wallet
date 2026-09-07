// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct NFTAssetDetails: Equatable, Sendable {
    public let assetData: NFTAssetData
    public let isOwned: Bool

    public init(assetData: NFTAssetData, isOwned: Bool) {
        self.assetData = assetData
        self.isOwned = isOwned
    }
}
