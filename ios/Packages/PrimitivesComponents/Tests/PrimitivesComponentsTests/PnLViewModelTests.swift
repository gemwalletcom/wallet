// Copyright (c). Gem Wallet. All rights reserved.

@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import Style
import Testing

struct PnLViewModelTests {
    @Test
    func textPnL() {
        #expect(PnLViewModel.mock(pnl: 500, marginAmount: 1000).text == "+$500.00 (+50.00%)")
        #expect(PnLViewModel.mock(pnl: -200, marginAmount: 1000).text == "-$200.00 (-20.00%)")
        #expect(PnLViewModel.mock(pnl: nil, marginAmount: 1000).text == nil)
    }

    @Test
    func percent() {
        #expect(PnLViewModel.mock(pnl: 100, marginAmount: 1000).percent == 10.0)
        #expect(PnLViewModel.mock(pnl: -50, marginAmount: 1000).percent == -5.0)
        #expect(PnLViewModel.mock(pnl: 0, marginAmount: 1000).percent == 0.0)
        #expect(PnLViewModel.mock(pnl: 100, marginAmount: 0).percent == 0.0)
        #expect(PnLViewModel.mock(pnl: nil, marginAmount: 1000).percent == 0.0)
    }

    @Test
    func color() {
        #expect(PnLViewModel.mock(pnl: 500, marginAmount: 1000).color == Colors.green)
        #expect(PnLViewModel.mock(pnl: -200, marginAmount: 1000).color == Colors.red)
        #expect(PnLViewModel.mock(pnl: 0, marginAmount: 1000).color == Colors.gray)
        #expect(PnLViewModel.mock(pnl: nil, marginAmount: 1000).color == .secondary)
    }
}
