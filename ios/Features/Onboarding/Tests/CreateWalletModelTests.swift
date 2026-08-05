// Copyright (c). Gem Wallet. All rights reserved.

import Keystore
import KeystoreTestKit
@testable import Onboarding
import Preferences
import StoreTestKit
import Testing
import WalletServiceTestKit
import WalletSessionService
import WalletSessionServiceTestKit

@MainActor
struct CreateWalletModelTests {
    @Test
    func createWalletSetsWalletConfiguration() async throws {
        let model = CreateWalletModel(
            walletService: .mock(keystore: KeystoreMock()),
            walletSessionService: WalletSessionService.mock(),
            avatarService: .init(store: .mock()),
            onComplete: nil,
        )

        let wallet = try await model.createWallet(words: LocalKeystore.words)
        let preferences = WalletPreferences(walletId: wallet.id)

        #expect(preferences.completeInitialWalletConfiguration)
        #expect(preferences.completeInitialLoadAssets)
        #expect(preferences.completeInitialLoadTransactions)
        #expect(preferences.completeInitialLoadNFTs)

        preferences.clear()
    }

    @Test
    func generateSecretPhraseReturnsGeneratedWords() {
        let model = CreateWalletModel(
            walletService: .mock(keystore: KeystoreMock()),
            walletSessionService: WalletSessionService.mock(),
            avatarService: .init(store: .mock()),
            onComplete: nil,
        )

        let words = model.generateSecretPhrase()

        #expect(words.isNotEmpty)
        #expect(words == LocalKeystore.words)
    }
}
