// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemPaymentService
import GemstonePrimitivesTestKit

public extension GemPaymentService {
    static func mock() -> GemPaymentService {
        GemPaymentService(provider: StubAlienProvider(), assets: .mock(), balance: .mock())
    }
}
