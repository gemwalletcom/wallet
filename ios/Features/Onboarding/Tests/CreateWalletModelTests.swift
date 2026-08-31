// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemOnboardingService
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
@testable import Onboarding
import Preferences
import PreferencesTestKit
import Primitives
import Store
import StoreTestKit
import Testing

@MainActor
struct CreateWalletModelTests {
    @Test
    func createWalletSetsWalletConfiguration() async throws {
        let walletStore = WalletStore.mock(db: .mockWithChains(AssetConfiguration.allChains))
        let model = CreateWalletModel(
            service: GemOnboardingService.mock(walletStore: walletStore),
            preferences: .mock(),
            onComplete: nil,
        )

        let wallet = try await model.createWallet(words: LocalKeystore.words)
        #expect(wallet.source == .create)
    }

    @Test
    func generateSecretPhraseReturnsGeneratedWords() {
        let model = CreateWalletModel(
            service: GemOnboardingService.mock(),
            preferences: .mock(),
            onComplete: nil,
        )

        let words = model.generateSecretPhrase()

        #expect(words.count == 12)
        #expect(words != model.generateSecretPhrase())
    }
}
