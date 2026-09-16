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
struct CreateWalletModelTests {
    @Test
    func createWalletSetsWalletConfiguration() async throws {
        let model = CreateWalletModel.mock(service: GemWalletService.mock(db: .mockWithChains(AssetConfiguration.allChains)))

        let created = try await model.createWallet(words: LocalKeystore.words)
        #expect(created.wallet.source == .create)
        #expect(created.hasExistingWallets == false)
    }

    @Test
    func generateSecretPhraseReturnsGeneratedWords() {
        let model = CreateWalletModel.mock()

        let words = model.generateSecretPhrase()

        #expect(words.count == 12)
        #expect(words != model.generateSecretPhrase())
    }
}
