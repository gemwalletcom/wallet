// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.paymentDecodeUrl
import Primitives

public extension Primitives.Payment {
    static func decode(_ string: String) throws -> Primitives.Payment {
        try Primitives.Payment(paymentDecodeUrl(string: string))
    }
}
