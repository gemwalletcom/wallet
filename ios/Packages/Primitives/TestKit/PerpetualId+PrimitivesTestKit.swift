// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension PerpetualId {
    static func mock(
        provider: PerpetualProvider = .hypercore,
        symbol: String = "BTC",
    ) -> PerpetualId {
        PerpetualId(provider: provider, symbol: symbol)
    }
}
