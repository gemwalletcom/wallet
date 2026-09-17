// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import Settings

public final class CurrencyStorageMock: CurrencyStorable, @unchecked Sendable {
    public var currency: Currency

    public init(currency: Currency = .usd) {
        self.currency = currency
    }
}
