// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
@testable import Primitives
import Testing

final class IntegerFormatterTests {
    @Test
    func stringDouble() {
        let formatter = IntegerFormatter()
        #expect(formatter.string(12.12) == "12")
        #expect(formatter.string(21, symbol: "BTC") == "21 BTC")
    }
}
