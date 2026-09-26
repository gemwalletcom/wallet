// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import class Gemstone.GemNameService
import class Gemstone.GemWalletService
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
@testable import Onboarding
import OnboardingTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct AcceptTermsViewModelTests {
    @Test
    func theTermsComeFromCoreAndStartUnconfirmed() {
        let model = AcceptTermsViewModel(preferences: .mock(), onNext: nil)

        #expect(model.items.isNotEmpty)
        #expect(model.isConfirmed == false)
        #expect(model.state.isNoData)
    }

    @Test
    func continuingRecordsTheAcceptance() {
        let preferences = ObservablePreferences.mock()
        let model = AcceptTermsViewModel(preferences: preferences, onNext: nil)

        #expect(preferences.isAcceptTermsCompleted == false)

        model.accept()

        #expect(preferences.isAcceptTermsCompleted)
    }

    @Test
    func everyTermMustBeTickedBeforeContinuing() {
        let model = AcceptTermsViewModel(preferences: .mock(), onNext: nil)

        for item in model.items.dropLast() {
            item.isConfirmed = true
        }
        #expect(model.isConfirmed == false)

        model.items.forEach { $0.isConfirmed = true }
        #expect(model.isConfirmed)
        #expect(model.state.isNoData == false)
    }
}

struct TermItemViewModelTests {
    @Test
    func tickingATermChangesHowItReads() {
        let item = TermItemViewModel(message: "I understand")

        let unconfirmed = item.style.color
        item.isConfirmed = true

        #expect(item.id == "I understand")
        #expect(item.style.color != unconfirmed)
    }
}

struct ImportWalletTypeViewModelTests {
    @Test
    func anEmptyQueryOffersEveryChain() {
        let model = ImportWalletTypeViewModel()

        #expect(model.items(for: "").isNotEmpty)
    }

    @Test
    func aQueryNarrowsTheChains() {
        let model = ImportWalletTypeViewModel()

        let all = model.items(for: "")
        let filtered = model.items(for: "bitcoin")

        #expect(filtered.isNotEmpty)
        #expect(filtered.count < all.count)
        #expect(filtered.contains(.bitcoin))
    }

    @Test
    func anUnknownQueryOffersNothing() {
        #expect(ImportWalletTypeViewModel().items(for: "zzzzzz").isEmpty)
    }
}
