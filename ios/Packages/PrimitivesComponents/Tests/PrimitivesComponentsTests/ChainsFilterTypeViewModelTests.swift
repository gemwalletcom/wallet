// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemChainsFilterSummary
import Localization
import Primitives
@testable import PrimitivesComponents
import Testing

struct ChainsFilterTypeViewModelTests {
    @Test
    func noSelectionReadsAsAll() {
        let model = ChainsFilterTypeViewModel(summary: .all)

        #expect(model.value == Localized.Common.all)
    }

    @Test
    func oneChainReadsAsItsName() {
        let model = ChainsFilterTypeViewModel(summary: .chain(chain: Chain.ethereum.rawValue))

        #expect(model.value == "Ethereum")
    }

    @Test
    func severalChainsReadAsACount() {
        let model = ChainsFilterTypeViewModel(summary: .count(count: 3))

        #expect(model.value == "3")
    }
}
