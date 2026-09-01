// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives
import struct Gemstone.GemTransferData

public enum PaymentDestination: Identifiable, Sendable {
    case confirm(GemTransferData)
    case recipient(SelectedAssetInput)
    case selectAsset(SelectAssetType, chains: [Chain])

    public var id: String {
        switch self {
        case let .confirm(data): "confirm-\(data.id)"
        case let .recipient(input): "recipient-\(input.id)"
        case let .selectAsset(type, _): "selectAsset-\(type.id)"
        }
    }
}
