// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
@testable import Onboarding
import OnboardingTestKit
import Primitives
import Store
import StoreTestKit
import Testing

@MainActor
struct CreateWalletViewModelTests {
    @Test
    func aCreatedWalletIsStoredAsCreated() async throws {
        let service = GemWalletService.mock(db: .mock(chains: AssetConfiguration.allChains))
        let model = CreateWalletViewModel.mock(service: service)

        try await model.createWallet(words: LocalKeystore.words)

        #expect(try await service.wallets().map(\.source) == [.create])
    }

    @Test
    func generatingASecretPhraseKeepsTheNewWords() throws {
        let model = CreateWalletViewModel.mock()

        try model.generateSecretPhrase()
        let words = model.words
        try model.generateSecretPhrase()

        #expect(words.count == 12)
        #expect(words != model.words)
    }
}
