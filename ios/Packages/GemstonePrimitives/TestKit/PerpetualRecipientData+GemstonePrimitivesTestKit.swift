// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemPerpetualPositionAction
import struct Gemstone.GemPerpetualTransferData
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension PerpetualRecipientData {
    static func mock(
        recipient: RecipientData = .mock(),
        positionAction: GemPerpetualPositionAction = .open(data: .mock()),
    ) -> PerpetualRecipientData {
        PerpetualRecipientData(
            recipient: recipient,
            positionAction: positionAction,
        )
    }
}

public extension GemPerpetualTransferData {
    static func mock(
        direction: PerpetualDirection = .long,
        asset: Asset = .mock(),
        leverage: UInt8 = 3,
    ) -> GemPerpetualTransferData {
        GemPerpetualTransferData(
            provider: PerpetualProvider.hypercore.map(),
            direction: direction.json(),
            asset: asset.map(),
            baseAsset: Asset.mock().map(),
            assetIndex: 0,
            price: 100.0,
            leverage: leverage,
            marginType: PerpetualMarginType.cross.json(),
        )
    }
}
