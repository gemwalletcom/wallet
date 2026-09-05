// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemPaymentRecipient
import Primitives

public struct SelectedAssetInput: Sendable, Hashable, Identifiable {
    public let type: SelectedAssetType
    public let assetData: AssetData
    public let recipient: GemPaymentRecipient?

    public init(type: SelectedAssetType, assetData: AssetData, recipient: GemPaymentRecipient? = .none) {
        self.type = type
        self.assetData = assetData
        self.recipient = recipient
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
