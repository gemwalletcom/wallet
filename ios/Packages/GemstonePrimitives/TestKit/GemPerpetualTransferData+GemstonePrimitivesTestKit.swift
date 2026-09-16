// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemPerpetualTransferData
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension GemPerpetualTransferData {
    static func mock(
        direction: PerpetualDirection = .long,
        asset: Asset = .mock(),
        leverage: UInt8 = 3,
    ) -> GemPerpetualTransferData {
        GemPerpetualTransferData(
            provider: PerpetualProvider.hypercore.toGem(),
            direction: direction.toGem(),
            asset: asset.toGem(),
            baseAsset: Asset.mock().toGem(),
            assetIndex: 0,
            price: 100.0,
            leverage: leverage,
            marginType: PerpetualMarginType.cross.toGem(),
        )
    }
}
