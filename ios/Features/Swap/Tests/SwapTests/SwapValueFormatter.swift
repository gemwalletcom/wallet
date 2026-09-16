// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import GemstonePrimitives
import Primitives
@testable import Swap
import Testing

struct SwapValueFormatterTests {
    let usFormatter = SwapValueFormatter(valueFormatter: ValueFormatter(locale: .US, style: .full))

    @Test
    func formatInputValueRejectsZero() throws {
        #expect(throws: SwapQuoteInputError.self) {
            try usFormatter.format(inputValue: "0", decimals: 8)
        }
    }

    @Test
    func formatValue() {
        #expect(usFormatter.format(value: BigInt(123_456_789), decimals: 6) == "123.456789")
    }
}
