// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemConfirmationProtocol
import Primitives

public extension GemConfirmationProtocol {
    var currency: Primitives.Currency {
        getCurrency().toPrimitives()
    }
}
