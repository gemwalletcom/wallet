// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Onboarding
import Primitives
import Testing

struct ImportWalletTypeSceneViewModelTests {
    @Test
    func anEmptyQueryOffersEveryChain() {
        let model = ImportWalletTypeSceneViewModel()

        #expect(model.items(for: "").isNotEmpty)
    }

    @Test
    func aQueryNarrowsTheChains() {
        let model = ImportWalletTypeSceneViewModel()

        let all = model.items(for: "")
        let filtered = model.items(for: "bitcoin")

        #expect(filtered.isNotEmpty)
        #expect(filtered.count < all.count)
        #expect(filtered.contains(.bitcoin))
    }

    @Test
    func anUnknownQueryOffersNothing() {
        #expect(ImportWalletTypeSceneViewModel().items(for: "zzzzzz").isEmpty)
    }
}
