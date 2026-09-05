// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemWalletService
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
        let model = CreateWalletModel(
            service: GemWalletService.mock(db: .mockWithChains(AssetConfiguration.allChains)),
            preferences: .mock(),
            avatarService: GemAvatarServiceMock(),
            onComplete: nil,
        )

        let created = try await model.createWallet(words: LocalKeystore.words)
        #expect(created.wallet.source == .create)
        #expect(created.hasExistingWallets == false)
    }

    @Test
    func generateSecretPhraseReturnsGeneratedWords() {
        let model = CreateWalletModel(
            service: GemWalletService.mock(),
            preferences: .mock(),
            avatarService: GemAvatarServiceMock(),
            onComplete: nil,
        )

        let words = model.generateSecretPhrase()

        #expect(words.count == 12)
        #expect(words != model.generateSecretPhrase())
    }
}
