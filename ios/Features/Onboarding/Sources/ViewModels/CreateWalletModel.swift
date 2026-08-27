// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAvatarServiceProtocol
import GemstoneServices
import Foundation
import GemstonePrimitives
import Preferences
import Primitives
import SwiftUI

@Observable
@MainActor
public final class CreateWalletModel {
    let walletService: WalletService
    let walletSessionService: any WalletSessionManageable
    let avatarService: any GemAvatarServiceProtocol
    let hasExistingWallets: Bool
    let onComplete: VoidAction

    var isPresentingSelectImageWallet: Wallet?

    public init(
        walletService: WalletService,
        walletSessionService: any WalletSessionManageable,
        avatarService: any GemAvatarServiceProtocol,
        onComplete: VoidAction,
    ) {
        self.walletService = walletService
        self.walletSessionService = walletSessionService
        self.avatarService = avatarService
        self.onComplete = onComplete
        hasExistingWallets = walletSessionService.wallets.isNotEmpty
    }

    public var isAcceptTermsCompleted: Bool {
        walletService.isAcceptTermsCompleted
    }

    func dismiss() {
        onComplete?()
    }
}

// MARK: - Actions

extension CreateWalletModel {
    func presentSelectImage(wallet: Wallet) {
        isPresentingSelectImageWallet = wallet
    }

    func generateSecretPhrase() -> [String] {
        do {
            return try walletService.createWallet()
        } catch {
            fatalError("Unable to create wallet")
        }
    }

    func createWallet(words: [String]) async throws -> Wallet {
        let result = try await walletService.importWallet(
            name: await WalletNameGenerator(type: .multicoin, walletService: walletService).name(),
            type: .phrase(words: words, chains: AssetConfiguration.allChains),
            source: .create,
        )
        walletService.acceptTerms()
        return result.wallet
    }

    func setupWalletComplete(wallet: Wallet) async {
        dismiss()
        await walletSessionService.setCurrent(wallet: wallet)
    }
}
