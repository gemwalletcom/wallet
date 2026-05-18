// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Testing

struct NumericFormatterTests {
    let formatter = NumericFormatter(locale: .US)
    let ua = NumericFormatter(locale: .RU_UA)

    @Test
    func decimal() {
        #expect(formatter.string(0) == "0.00")
        #expect(formatter.string(11.12) == "11.12")
        #expect(formatter.string(11) == "11.00")
        #expect(formatter.string(12_000_123) == "12,000,123.00")

        #expect(formatter.string(0.12) == "0.12")
        #expect(formatter.string(0.00012) == "0.00012")
        #expect(formatter.string(0.0000000002) == "0.0000000002")
        #expect(formatter.string(0.0000000000001) == "0.00")
    }

    @Test
    func withSymbol() {
        #expect(formatter.string(1234.56, symbol: "BTC") == "1,234.56 BTC")
        #expect(formatter.string(0.0001234, symbol: "BTC") == "0.0001234 BTC")
        #expect(formatter.string(0, symbol: "BTC") == "0.00 BTC")
    }

    @Test
    func uaLocale() {
        #expect(ua.string(29.73) == "29,73")
    }
}
