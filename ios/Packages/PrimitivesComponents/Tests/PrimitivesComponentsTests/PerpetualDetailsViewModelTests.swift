// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import Testing

@MainActor
struct PerpetualDetailsViewModelTests {
    @Test
    func summaryReadsTheCoreText() {
        #expect(PerpetualDetailsViewModel.mock(.close(data: .mock(pnl: 500, marginAmount: 1000))).listItemModel.subtitle == "+$500.00 (+50.00%)")
        #expect(PerpetualDetailsViewModel.mock(.open(data: .mock())).listItemModel.subtitle == "LONG 3x")
        #expect(PerpetualDetailsViewModel.mock(.increase(data: .mock(direction: .short))).listItemModel.subtitle == "Increase Short")
        #expect(PerpetualDetailsViewModel.mock(.reduce(data: .mock(positionDirection: .short))).listItemModel.subtitle == "Reduce Short")
    }

    @Test
    func sectionsComeFromCore() {
        let model = PerpetualDetailsViewModel.mock(.open(data: .mock(entryPrice: 48000.0)))

        #expect(model.sections.count == 3, "position, amounts and prices; no autoclose section without a trigger order")
        #expect(model.sections.last?.values.count == 3)
    }
}
