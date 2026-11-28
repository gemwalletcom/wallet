// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemPaymentRecipient
import enum Gemstone.GemSelectAssetType
import Primitives

public struct SelectAssetInput: Hashable {
    public let type: GemSelectAssetType
    public let assetData: AssetData
    public let recipient: GemPaymentRecipient?

    public init(type: GemSelectAssetType, assetData: AssetData, recipient: GemPaymentRecipient? = .none) {
        self.type = type
        self.assetData = assetData
        self.recipient = recipient
    }
}

public extension SelectAssetInput {
    var asset: Asset {
        assetData.asset
    }

    var assetAddress: AssetAddress {
        assetData.assetAddress
    }
}
