// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemPaymentService
import Primitives

public extension Primitives.Payment {
    static func decode(_ string: String, paymentService: GemPaymentService) throws -> Primitives.Payment {
        try Primitives.Payment(paymentService.decodeUrl(string: string))
    }
}
