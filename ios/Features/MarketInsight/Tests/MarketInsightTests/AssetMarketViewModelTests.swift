// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing

@testable import MarketInsight

struct AssetMarketViewModelTests {
    @Test
    func withValuesFiltersSubtitles() {
        let items = [
            MarketValueViewModel(title: "A", subtitle: "Value"),
            MarketValueViewModel(title: "B", subtitle: ""),
            MarketValueViewModel(title: "B", subtitle: nil),
        ]

        #expect(items.withValues().count == 1)
    }
}
