// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemWalletServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class CreateWalletModel {
    private let service: any GemWalletServiceProtocol

    func verifyPhraseModel(onComplete: @escaping ([String]) async throws -> Void) -> VerifyPhraseViewModel {
        VerifyPhraseViewModel(
            session: service.verifyPhraseSession(words: words),
            onComplete: onComplete,
        )
    }

    let preferences: ObservablePreferences
    let onComplete: VoidAction

    private(set) var words: [String] = []
    var isPresentingSelectImageWallet: Wallet?
    var isPresentingAlertMessage: AlertMessage?

    public init(
        service: any GemWalletServiceProtocol,
        preferences: ObservablePreferences,
        onComplete: VoidAction,
    ) {
        self.service = service
        self.preferences = preferences
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
        WalletImageViewModel(wallet: wallet, source: .onboarding, service: service)
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

    func generateSecretPhrase() throws {
        words = try service.createWallet()
    }

    func createWallet(words: [String]) async throws -> CreatedWallet {
        let name = try await service.defaultWalletName(chain: .none)
        let result = try await service.importWallet(
            name: name.text.text,
            type: service.importRequest(kind: .phrase, chain: nil, input: words.joined(separator: " "), nameRecord: nil),
            source: .create,
        )
        return CreatedWallet(wallet: result.wallet, hasExistingWallets: name.hasExistingWallets)
    }

    func setupWalletComplete(wallet: Wallet) {
        do {
            try service.setCurrentWalletId(walletId: wallet.id.id)
            dismiss()
        } catch {
            isPresentingAlertMessage = AlertMessage(title: Localized.Errors.errorOccurred, error: error)
        }
    }
}

struct CreatedWallet {
    let wallet: Wallet
    let hasExistingWallets: Bool
}
