// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives
@testable import PrimitivesComponents
import Testing

struct ChainsFilterTypeViewModelTests {
    @Test
    func noSelectionReadsAsAll() {
        let model = ChainsFilterTypeViewModel(type: ChainsFilterType(selectedChains: []))

        #expect(model.value == Localized.Common.all)
    }

    @Test
    func oneChainReadsAsItsName() {
        let model = ChainsFilterTypeViewModel(type: ChainsFilterType(selectedChains: [.ethereum]))

        #expect(model.value == "Ethereum")
    }

    @Test
    func severalChainsReadAsACount() {
        let model = ChainsFilterTypeViewModel(type: ChainsFilterType(selectedChains: [.ethereum, .bitcoin, .solana]))

        #expect(model.value == "3")
    }
}
