// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.perpetualConfirmDetails
import enum Gemstone.PerpetualType
import GemstonePrimitivesTestKit
import PrimitivesComponents

public extension PerpetualDetailsViewModel {
    static func mock(_ type: PerpetualType = .open(data: .mock(direction: .long, price: "100", fiatValue: 100, size: "1", slippage: 2, leverage: 3, marketPrice: 100, marginAmount: 33.33))) -> PerpetualDetailsViewModel {
        PerpetualDetailsViewModel(details: perpetualConfirmDetails(perpetualType: type)!)
    }
}
