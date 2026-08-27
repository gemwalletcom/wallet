// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import Primitives
import GemstoneServices
import GemstoneServicesTestKit
@testable import Onboarding
import Preferences
import Store
import StoreTestKit
import Testing
import GemstonePrimitives

@MainActor
struct CreateWalletModelTests {
    @Test
    func createWalletSetsWalletConfiguration() async throws {
        let walletStore = WalletStore.mock(db: .mockWithChains(AssetConfiguration.allChains))
        let model = CreateWalletModel(
            walletService: .mock(walletStore: walletStore),
            walletSessionService: WalletSessionService.mock(store: walletStore),
            avatarService: GemAvatarServiceMock(),
            onComplete: nil,
        )

        let wallet = try await model.createWallet(words: LocalKeystore.words)
        #expect(wallet.source == .create)
    }

    @Test
    func generateSecretPhraseReturnsGeneratedWords() {
        let model = CreateWalletModel(
            walletService: .mock(),
            walletSessionService: WalletSessionService.mock(),
            avatarService: GemAvatarServiceMock(),
            onComplete: nil,
        )

        let words = model.generateSecretPhrase()

        #expect(words.count == 12)
        #expect(words != model.generateSecretPhrase())
    }
}
