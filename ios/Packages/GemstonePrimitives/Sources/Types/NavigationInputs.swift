// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemPaymentRecipient
import struct Gemstone.GemTransferData
import Primitives

public typealias AmountInputAction = ((AmountInput) -> Void)?

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

public struct AmountInput: Hashable, Identifiable {
    public let type: AmountType
    public let asset: Asset

    public init(type: AmountType, asset: Asset) {
        self.type = type
        self.asset = asset
    }

    public var id: String {
        asset.id.identifier
    }
}

public struct ConfirmTransferInput: Hashable {
    public let data: GemTransferData

    public init(data: GemTransferData) {
        self.data = data
    }
}
