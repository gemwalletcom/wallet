// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.perpetualDetails
import enum Gemstone.PerpetualType
import GemstonePrimitivesTestKit
import PrimitivesComponents

public extension PerpetualDetailsViewModel {
    static func mock(_ type: PerpetualType = .open(data: .mock())) -> PerpetualDetailsViewModel {
        PerpetualDetailsViewModel(details: perpetualDetails(perpetualType: type)!)
    }
}
