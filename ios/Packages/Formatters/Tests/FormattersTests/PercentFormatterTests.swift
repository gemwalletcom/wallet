// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Testing

struct PercentFormatterTests {
    let signedUS = PercentFormatter(locale: .US, sign: .signed)
    let signedUK = PercentFormatter(locale: .UK, sign: .signed)
    let unsignedUS = PercentFormatter(locale: .US, sign: .unsigned)

    @Test
    func signed() {
        #expect(signedUS.string(-1.23) == "-1.23%")
        #expect(signedUS.string(0) == "+0.00%")
        #expect(signedUS.string(11.12) == "+11.12%")
        #expect(signedUS.string(11) == "+11.00%")
        #expect(signedUS.string(12_000_123) == "+12,000,123.00%")

        #expect(signedUK.string(-1.23) == "-1.23%")
        #expect(signedUK.string(11.12) == "+11.12%")
        #expect(signedUK.string(12_000_123) == "+12,000,123.00%")
    }

    @Test
    func unsigned() {
        #expect(unsignedUS.string(-1.23) == "1.23%")
        #expect(unsignedUS.string(11.12) == "11.12%")
        #expect(unsignedUS.string(11) == "11.00%")
        #expect(unsignedUS.string(12_000_123) == "12,000,123.00%")
    }
}
