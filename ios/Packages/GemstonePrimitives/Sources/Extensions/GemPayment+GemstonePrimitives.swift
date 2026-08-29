// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemPaymentService
import Primitives

public extension Primitives.Payment {
    static func decode(_ string: String) throws -> Primitives.Payment {
        try Primitives.Payment(GemPaymentService().decodeUrl(string: string))
    }
}
