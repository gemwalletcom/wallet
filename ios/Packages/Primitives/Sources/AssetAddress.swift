// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct AssetAddress: Codable, Equatable, Hashable, Sendable {
    public let asset: Asset
    public let address: String

    public init(asset: Asset, address: String) {
        self.asset = asset
        self.address = address
    }
}
