// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct SelectedAssetInput: Sendable, Hashable, Identifiable {
    public let type: SelectedAssetType
    public let assetData: AssetData

    public init(type: SelectedAssetType, assetData: AssetData) {
        self.type = type
        self.assetData = assetData
    }

    public var id: String {
        type.id
    }

    public var asset: Asset {
        assetData.asset
    }

    public var assetAddress: AssetAddress {
        assetData.assetAddress
    }
}
