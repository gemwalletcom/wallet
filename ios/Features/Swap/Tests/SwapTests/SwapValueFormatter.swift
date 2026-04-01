// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Primitives
import Testing

@testable import Swap

struct SwapValueFormatterTests {
    let usFormatter = SwapValueFormatter(valueFormatter: ValueFormatter(locale: .US, style: .full))

    @Test
    func formatInputValueValid_USFull() throws {
        #expect(try usFormatter.format(inputValue: "0.12345", decimals: 8) == BigInt(12_345_000))
    }

    @Test
    func formatInputValueInvalidZero_USFull() throws {
        #expect(throws: SwapQuoteInputError.self) {
            try usFormatter.format(inputValue: "0", decimals: 8)
        }
    }

    @Test
    func formatQuoteValue_USFull() throws {
        #expect(
            try usFormatter.format(quoteValue: "1000000000000000000", decimals: 18) == "1",
        )
    }

    @Test
    func formatValue_USFull() {
        #expect(usFormatter.format(value: BigInt(123_456_789), decimals: 6) == "123.456789")
    }

    @Test
    func formatInputValueValid_UAFull() throws {
        let swapFormatter = SwapValueFormatter(valueFormatter: ValueFormatter(locale: .UA, style: .full))
        #expect(try swapFormatter.format(inputValue: "0,12345", decimals: 8) == BigInt(12_345_000))
    }

    @Test
    func formatInputValueValid_BRFull() throws {
        let swapFormatter = SwapValueFormatter(valueFormatter: ValueFormatter(locale: .PT_BR, style: .full))
        #expect(try swapFormatter.format(inputValue: "0,12345", decimals: 8) == BigInt(12_345_000))
    }

    @Test
    func formatInputValueValid_FRFull() throws {
        let swapFormatter = SwapValueFormatter(valueFormatter: ValueFormatter(locale: .FR, style: .full))
        #expect(try swapFormatter.format(inputValue: "0,12345", decimals: 8) == BigInt(12_345_000))
    }

    @Test
    func amountToDecimalConversion() throws {
        #expect(try usFormatter.format(inputValue: "123.123", decimals: 3) == BigInt(123_123))
        #expect(try usFormatter.format(inputValue: "0.000000000000004162", decimals: 18) == BigInt(4162))
    }
}
