// Copyright (c). Gem Wallet. All rights reserved.

import PrimitivesComponents
import Testing

struct AprViewModelTests {
    @Test
    func subtitle() {
        #expect(AprViewModel(apr: 13.5).subtitle.text == "13.50%")
        #expect(AprViewModel(apr: .zero).subtitle.text == .empty)
    }

    @Test
    func text() {
        #expect(AprViewModel(apr: 2.15).text == "APR 2.15%")
        #expect(AprViewModel(apr: .zero).text == "APR ")
    }
}
