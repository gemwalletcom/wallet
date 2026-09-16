// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import Testing

struct PerpetualDetailsViewModelTests {
    @Test
    func leverageText() {
        #expect(PerpetualDetailsViewModel.mock(.open(data: .mock(leverage: 5))).leverageText == "5x")
    }

    @Test
    func slippageField() {
        #expect(PerpetualDetailsViewModel.mock(.open(data: .mock(slippage: 2.0))).slippageField.value.text == "2.00%")
    }

    @Test
    func entryPriceField() {
        #expect(PerpetualDetailsViewModel.mock(.open(data: .mock(entryPrice: 48000.0))).entryPriceField?.value.text == "$48,000.00")
        #expect(PerpetualDetailsViewModel.mock(.open(data: .mock(entryPrice: nil))).entryPriceField == nil)
    }

    @Test
    func marginField() {
        #expect(PerpetualDetailsViewModel.mock(.open(data: .mock(marginAmount: 1000.0))).marginField.value.text == "$1,000.00")
    }

    @Test
    func sizeField() {
        #expect(PerpetualDetailsViewModel.mock(.open(data: .mock(fiatValue: 5000.0))).sizeField.value.text == "$5,000.00")
    }

    @Test
    func positionText() {
        #expect(PerpetualDetailsViewModel.mock(.open(data: .mock(direction: .long, leverage: 40))).positionText == "Long 40x")
        #expect(PerpetualDetailsViewModel.mock(.open(data: .mock(direction: .short, leverage: 10))).positionText == "Short 10x")
        #expect(PerpetualDetailsViewModel.mock(.increase(data: .mock(direction: .long, leverage: 5))).positionText == "Long 5x")

        let reduceModel = PerpetualDetailsViewModel.mock(.reduce(data: .mock(positionDirection: .short)))
        #expect(reduceModel.positionText == "Short 3x")
    }

    @Test
    func listItemModelSubtitle() {
        let closeModel = PerpetualDetailsViewModel.mock(.close(data: .mock(pnl: 500, marginAmount: 1000)))
        #expect(closeModel.listItemModel.title == "Details")
        #expect(closeModel.listItemModel.subtitle == "+$500.00 (+50.00%)")

        #expect(PerpetualDetailsViewModel.mock(.open(data: .mock())).listItemModel.subtitle == "Long 3x")

        let increaseModel = PerpetualDetailsViewModel.mock(.increase(data: .mock()))
        #expect(increaseModel.listItemModel.subtitle == "Increase Long")

        let increaseShortModel = PerpetualDetailsViewModel.mock(.increase(data: .mock(direction: .short)))
        #expect(increaseShortModel.listItemModel.subtitle == "Increase Short")

        let reduceLongModel = PerpetualDetailsViewModel.mock(.reduce(data: .mock()))
        #expect(reduceLongModel.listItemModel.subtitle == "Reduce Long")

        let reduceShortModel = PerpetualDetailsViewModel.mock(.reduce(data: .mock(positionDirection: .short)))
        #expect(reduceShortModel.listItemModel.subtitle == "Reduce Short")
    }
}
