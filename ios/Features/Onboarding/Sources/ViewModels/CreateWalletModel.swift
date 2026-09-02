// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemOnboardingServiceProtocol
import GemstoneServices
import Foundation
import GemstonePrimitives
import Preferences
import Primitives
import SwiftUI

@Observable
@MainActor
public final class CreateWalletModel {
    private let service: any GemOnboardingServiceProtocol
    private let preferences: ObservablePreferences
    private let walletImage: @MainActor (Wallet) -> WalletImageViewModel
    let hasExistingWallets: Bool
    let onComplete: VoidAction

    var isPresentingSelectImageWallet: Wallet?

    public init(
        service: any GemOnboardingServiceProtocol,
        preferences: ObservablePreferences,
        walletImage: @escaping @MainActor (Wallet) -> WalletImageViewModel,
        onComplete: VoidAction,
    ) {
        self.service = service
        self.preferences = preferences
        self.walletImage = walletImage
        self.onComplete = onComplete
        hasExistingWallets = ((try? service.getWallets()) ?? []).isNotEmpty
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
        walletImage(wallet)
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
        let result = try await service.importWallet(
            name: await WalletNameGenerator(type: .multicoin, service: service).name(),
            type: .phrase(words: words, chains: AssetConfiguration.allChains),
            source: .create,
        )
        preferences.acceptTerms()
        return result.wallet
    }

    func setupWalletComplete(wallet: Wallet) async {
        dismiss()
        do {
            try service.setCurrentWallet(walletId: wallet.id.id)
        } catch {
            debugLog("set current wallet error: \(error)")
        }
    }
}
