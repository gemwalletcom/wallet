// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemPaymentService
import Primitives

private let paymentService = GemPaymentService()

public extension Primitives.Payment {
    static func decode(_ string: String) throws -> Primitives.Payment {
        try Primitives.Payment(paymentService.decodeUrl(string: string))
    }
}
