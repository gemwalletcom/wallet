// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
@testable import Formatters
import Foundation
import Testing

final class BigNumberFormatterTests {
    let formatter = BigNumberFormatter.standard

    @Test
    func fromBigInt() {
        #expect(formatter.string(from: BigInt(10000), decimals: 2) == "100")
        #expect(formatter.string(from: BigInt(123_456_789), decimals: 4) == "12,345.6789")
        #expect(formatter.decimal(from: BigInt(12317), decimals: 8) == Decimal(string: "0.00012317"))
        #expect(formatter.double(from: BigInt(12317), decimals: 8) == 0.00012317)
    }

    @Test
    func fromBigIntEULocalization() {
        let formatter = BigNumberFormatter(locale: Locale.RU_UA)
        #expect(formatter.string(from: BigInt(12_317_000), decimals: 8) == "0,12317")
    }
}
