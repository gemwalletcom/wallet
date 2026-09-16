// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemChartServiceProtocol
import Primitives

public extension GemChartServiceProtocol {
    var currency: Primitives.Currency {
        getCurrency().toPrimitives()
    }
}
