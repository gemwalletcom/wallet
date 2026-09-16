// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
@testable import GemstonePrimitives
import Foundation
import Testing

final class ValueFormatterTests {
    @Test
    func testShort() {
        let formatter = ValueFormatter(locale: .US, style: .short)

        #expect(formatter.string(123, decimals: 0) == "123")
        #expect(formatter.string(12344, decimals: 6) == "0.0123")
        #expect(formatter.string(0, decimals: 0) == "0")

        #expect(formatter.string(1_000_000, decimals: 0) == "1M")
        #expect(formatter.string(1000, decimals: 0) == "1,000")
        #expect(formatter.string(100, decimals: 0) == "100")
        #expect(formatter.string(10, decimals: 0) == "10")
        #expect(formatter.string(1, decimals: 0) == "1")

        #expect(formatter.string(1992, decimals: 4) == "0.19")
        #expect(formatter.string(99999, decimals: 6) == "0.0999")
        #expect(formatter.string(1, decimals: 1) == "0.1")
        #expect(formatter.string(1, decimals: 2) == "0.01")
        #expect(formatter.string(1, decimals: 3) == "0.001")

        #expect(formatter.string(1, decimals: 4) == "0.0001")
        #expect(formatter.string(1, decimals: 5) == "<0.0001")
        #expect(formatter.string(1, decimals: 6) == "<0.0001")
        #expect(formatter.string(12_345_678_910, decimals: 6) == "12,345.67")

        #expect(formatter.string(7_758_980_129_936_940, decimals: 18, currency: "BNB") == "0.0077 BNB")
        #expect(formatter.string(2_737_071, decimals: 8, currency: "BTC") == "0.0273 BTC")
        #expect(formatter.string(629, decimals: 8, currency: "WBTC") == "<0.0001 WBTC")
    }

    @Test
    func testFull() {
        let formatter = ValueFormatter(locale: .US, style: .full)

        #expect(formatter.string(123, decimals: 0) == "123")
        #expect(formatter.string(12344, decimals: 6) == "0.012344")
        #expect(formatter.string(0, decimals: 0) == "0")

        #expect(formatter.string(1_000_000, decimals: 0) == "1,000,000")
        #expect(formatter.string(1000, decimals: 0) == "1,000")
        #expect(formatter.string(100, decimals: 0) == "100")
        #expect(formatter.string(10, decimals: 0) == "10")
        #expect(formatter.string(1, decimals: 0) == "1")

        #expect(formatter.string(1, decimals: 1) == "0.1")
        #expect(formatter.string(1, decimals: 2) == "0.01")
        #expect(formatter.string(1, decimals: 3) == "0.001")
        #expect(formatter.string(1, decimals: 4) == "0.0001")
        #expect(formatter.string(1, decimals: 5) == "0.00001")
        #expect(formatter.string(1, decimals: 6) == "0.000001")
        #expect(formatter.string(BigInt("12345678910111213"), decimals: 18) == "0.012345678910111213")
        #expect(formatter.string(BigInt("1"), decimals: 18) == "0.000000000000000001")
        #expect(formatter.string(BigInt("18761627355200464162"), decimals: 18) == "18.761627355200464162")
        #expect(formatter.string(BigInt("4162"), decimals: 18) == "0.000000000000004162")

        #expect(formatter.string(2_737_071, decimals: 8, currency: "BTC") == "0.02737071 BTC")
    }

    @Test
    func fromDouble() throws {
        let formatter = ValueFormatter(locale: .US, style: .full)

        #expect(try formatter.double(from: 122_131_233, decimals: 0) == Double(122_131_233.0))
    }

    @Test
    func testAuto() {
        let formatter = ValueFormatter(locale: .US, style: .auto)

        #expect(formatter.string(123, decimals: 0) == "123")
        #expect(formatter.string(12344, decimals: 6) == "0.01234")
        #expect(formatter.string(11_112_344, decimals: 10) == "0.001111")
        #expect(formatter.string(1, decimals: 4) == "0.0001")
        #expect(formatter.string(1, decimals: 5) == "0.00001")
        #expect(formatter.string(4162, decimals: 18) == "0.000000000000004162")
        #expect(formatter.string(9_646_202_573_492, decimals: 18) == "0.000009646")
        #expect(formatter.string(1, decimals: 18) == "0.000000000000000001")
    }
}
