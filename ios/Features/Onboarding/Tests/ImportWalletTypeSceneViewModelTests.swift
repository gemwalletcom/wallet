// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Onboarding
import Primitives
import Testing

struct ImportWalletTypeSceneViewModelTests {
    @Test
    func anEmptyQueryOffersEveryChain() {
        let model = ImportWalletTypeSceneViewModel()

        #expect(model.types(for: "").chains.isNotEmpty)
    }

    @Test
    func aQueryNarrowsTheChains() {
        let model = ImportWalletTypeSceneViewModel()

        let all = model.types(for: "").chains
        let filtered = model.types(for: "bitcoin").chains

        #expect(filtered.isNotEmpty)
        #expect(filtered.count < all.count)
        #expect(filtered.contains { $0.chain == Chain.bitcoin.rawValue })
    }

    @Test
    func anUnknownQueryOffersNothing() {
        #expect(ImportWalletTypeSceneViewModel().types(for: "zzzzzz").chains.isEmpty)
    }
}
