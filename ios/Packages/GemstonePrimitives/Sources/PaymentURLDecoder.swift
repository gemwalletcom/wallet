// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.paymentDecodeUrl
import Primitives

public enum PaymentURLDecoder {
    public static func decode(_ string: String) throws -> Payment {
        try paymentDecodeUrl(string: string).map()
    }
}
