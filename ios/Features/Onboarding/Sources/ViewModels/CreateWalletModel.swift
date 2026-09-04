// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemAvatarServiceProtocol
import protocol Gemstone.GemWalletServiceProtocol
import GemstoneServices
import Foundation
import GemstonePrimitives
import Preferences
import Primitives
import SwiftUI

@Observable
@MainActor
public final class CreateWalletModel {
    private let service: any GemWalletServiceProtocol
    private let preferences: ObservablePreferences
    private let avatarService: any GemAvatarServiceProtocol
    private(set) var hasExistingWallets: Bool = false
    let onComplete: VoidAction

    var isPresentingSelectImageWallet: Wallet?

    public init(
        service: any GemWalletServiceProtocol,
        preferences: ObservablePreferences,
        avatarService: any GemAvatarServiceProtocol,
        onComplete: VoidAction,
    ) {
        self.service = service
        self.preferences = preferences
        self.avatarService = avatarService
        self.onComplete = onComplete
    }

    public var isAcceptTermsCompleted: Bool {
        preferences.isAcceptTermsCompleted
    }

    func setupWalletModel(wallet: Wallet, onComplete: @escaping (Wallet) -> Void) -> SetupWalletViewModel {
        SetupWalletViewModel(
            wallet: wallet,
            service: service,
            onSelectImage: { [weak self] in self?.presentSelectImage(wallet: $0) },
            onComplete: onComplete,
        )
    }

    func walletImageModel(wallet: Wallet) -> WalletImageViewModel {
        WalletImageViewModel(wallet: wallet, source: .onboarding, avatarService: avatarService)
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
            return try service.createWallet()
        } catch {
            fatalError("Unable to create wallet")
        }
    }

    func createWallet(words: [String]) async throws -> Wallet {
        let generator = WalletNameGenerator(defaultName: try await service.defaultWalletName(chain: .none))
        hasExistingWallets = generator.hasExistingWallets
        let result = try await service.importWallet(
            name: generator.name(),
            type: .phrase(words: words, chains: AssetConfiguration.allChains),
            source: .create,
        )
        preferences.acceptTerms()
        return result.wallet
    }

    func setupWalletComplete(wallet: Wallet) async {
        dismiss()
        do {
            try service.setCurrentWalletId(walletId: wallet.id.id)
        } catch {
            debugLog("set current wallet error: \(error)")
        }
    }
}
