// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import PrimitivesComponents
import PrimitivesComponentsTestKit
import Style
import Testing

struct NumericViewModelTests {
    @Test
    func amountReadsInItsTone() {
        let incoming = NumericViewModel.mock(header: .mock(amount: .mock(value: 1, unit: .symbol(symbol: "BTC"), display: .number(precision: .fraction(min: 0, max: 2)), notation: .signed, tone: .positive, rounding: .toNearest)))
        let plain = NumericViewModel.mock()

        #expect(incoming.amount.text == "+1 BTC")
        #expect(incoming.amount.style.color == Colors.green)
        #expect(plain.amount.text == "1 BTC")
        #expect(plain.amount.style.color == Colors.black)
    }

    @Test
    func fiatText() {
        #expect(NumericViewModel.mock().fiat?.text == "$2.00")
        #expect(NumericViewModel.mock().fiat?.style.color == Colors.gray)
        #expect(NumericViewModel.mock(header: .mock(fiat: nil)).fiat == nil)
    }
}
