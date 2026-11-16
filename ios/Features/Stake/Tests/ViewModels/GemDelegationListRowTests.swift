// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemDelegationListRow
import GemstonePrimitivesTestKit
@testable import Stake
import Testing

struct GemDelegationListRowTests {
    @Test
    func itemShowsTheBalanceAndItsFiatValue() {
        let row = GemDelegationListRow.mock(
            balance: .mock(value: 1500, unit: .symbol(symbol: "TRX"), display: .number(precision: .fraction(min: 0, max: 2))),
            fiat: .mock(value: 3000, unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 2, max: 2))),
        )

        #expect(row.listItem.subtitle == "1,500 TRX")
        #expect(row.listItem.subtitleExtra == "$3,000.00")
    }
}
