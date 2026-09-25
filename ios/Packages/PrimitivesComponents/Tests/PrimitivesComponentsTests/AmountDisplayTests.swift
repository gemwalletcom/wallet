// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

struct AmountDisplayTests {
    let asset = Asset.mock()

    @Test
    func symbolFactory() {
        let display = AmountDisplay.symbol(asset: asset)

        guard case let .symbol(viewModel) = display else {
            Issue.record("Expected symbol display type")
            return
        }

        #expect(viewModel.amount.text == asset.symbol)
    }

    @Test
    func amountDisplayable() {
        let numericDisplay = AmountDisplay.numeric(.mock())
        let symbolDisplay = AmountDisplay.symbol(asset: asset)

        #expect(numericDisplay.amount.text == "1 BTC")
        #expect(symbolDisplay.amount.text == asset.symbol)
        #expect(numericDisplay.fiat?.text == "$2.00")
        #expect(symbolDisplay.fiat == nil)
    }
}
