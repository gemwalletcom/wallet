// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public struct SelectAssetInput: Hashable {
    public let type: SelectAssetType
    public let assetData: AssetData

    public init(type: SelectAssetType, assetData: AssetData) {
        self.type = type
        self.assetData = assetData
    }
}

extension SelectAssetInput: Identifiable {
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
