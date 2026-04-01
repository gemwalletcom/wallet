// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
@testable import Formatters
import Foundation
import Testing

final class BigNumberFormatterTests {
    let formatter = BigNumberFormatter.standard

    @Test
    func fromString() {
        #expect(throws: Never.self) {
            let result = try self.formatter.number(from: "0.00012317", decimals: 8)
            #expect(result == 12317)
        }
    }

    @Test
    func fromNumber() {
        #expect(formatter.number(from: 100_000, decimals: 7) == 1_000_000_000_000)
        #expect(formatter.number(from: 10, decimals: 0) == 10)
    }

    @Test
    func fromNumberEULocalization() {
        let formatter = BigNumberFormatter(locale: Locale.RU_UA)
        #expect(throws: Never.self) {
            let result = try formatter.number(from: "0,12317", decimals: 8)
            #expect(result == 12_317_000)
        }
    }

    @Test
    func fromBigInt() {
        #expect(formatter.string(from: BigInt(10000), decimals: 2) == "100")
    }
}
