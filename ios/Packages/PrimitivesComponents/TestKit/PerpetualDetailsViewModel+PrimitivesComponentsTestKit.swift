// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.perpetualConfirmDetails
import enum Gemstone.PerpetualType
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit

public extension PerpetualDetailsViewModel {
    static func mock(_ type: PerpetualType = .open(data: .mock(direction: .long, price: "100", fiatValue: 100, size: "1", slippage: 2, leverage: 3, marketPrice: 100, marginAmount: 33.33))) -> PerpetualDetailsViewModel {
        PerpetualDetailsViewModel(details: perpetualConfirmDetails(asset: Asset.mock(id: .mock(chain: .hyperCore), type: .perpetual).toGem(), perpetualType: type)!)
    }
}
