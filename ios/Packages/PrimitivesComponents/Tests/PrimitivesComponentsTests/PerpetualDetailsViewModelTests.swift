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
    }
}
