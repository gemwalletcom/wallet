// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Style
import Testing

struct NumericViewModelTests {
    @Test
    func amountText() {
        let viewModel = NumericViewModel.mock(price: .mock(price: 2.0), sign: .incoming)

        #expect(viewModel.amount.text == "+1 BTC")
        #expect(viewModel.amount.style.color == Colors.green)
    }

    @Test
    func amountTextOutgoing() {
        let viewModel = NumericViewModel.mock(price: .mock(price: 2.0), sign: .outgoing)

        #expect(viewModel.amount.text == "-1 BTC")
        #expect(viewModel.amount.style.color == Colors.black)
    }

    @Test
    func amountTextNoSign() {
        let viewModel = NumericViewModel.mock(price: .mock(price: 2.0))

        #expect(viewModel.amount.text == "1 BTC")
        #expect(viewModel.amount.style.color == Colors.black)
    }

    @Test
    func fiatText() {
        let viewModel = NumericViewModel.mock(price: .mock(price: 2.0), sign: .incoming)

        #expect(viewModel.fiat?.text == "$2.00")
        #expect(viewModel.fiat?.style.color == Colors.gray)
    }

    @Test
    func fiatTextNilWhenPriceIsNil() {
        let viewModel = NumericViewModel.mock(price: nil, sign: .incoming)

        #expect(viewModel.fiat == nil)
    }

    @Test
    func zeroValueNoSign() {
        let viewModel = NumericViewModel.mock(price: .mock(price: 2.0), value: .zero, sign: .incoming)

        #expect(viewModel.amount.text == "0 BTC")
    }
}
