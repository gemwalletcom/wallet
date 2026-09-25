// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemHeaderAmount
import GemstonePrimitivesTestKit
import PrimitivesComponents

public extension NumericViewModel {
    static func mock(header: GemHeaderAmount = .mock()) -> NumericViewModel {
        NumericViewModel(header: header)
    }
}
