// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

public struct CreateWalletNavigationStack: View {
    @State private var model: CreateWalletViewModel
    @State private var navigationPath = NavigationPath()

    public init(model: CreateWalletViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        NavigationStack(path: $navigationPath) {
            rootScene
                .toolbarDismissItem(type: .close, placement: .topBarLeading)
                .navigationBarTitleDisplayMode(.inline)
                .navigationDestination(for: Scenes.VerifyPhrase.self) { _ in
                    VerifyPhraseScene(
                        model: model.verifyPhraseModel(onComplete: onVerifyPhraseComplete),
                    )
                }
                .navigationDestination(for: Scenes.CreateWallet.self) { _ in
                    ShowSecretDataScene(
                        model: SecretDataViewModel(
                            secret: .words(words: model.words),
                            continueAction: { navigate(to: .verifyPhrase) },
                        ),
                    )
                }
                .navigationDestination(for: Scenes.SecurityReminder.self) { _ in
                    securityReminderScene
                }
                .alertSheet($model.isPresentingAlertMessage)
        }
    }

    @ViewBuilder
    private var rootScene: some View {
        if model.isAcceptTermsCompleted {
            securityReminderScene
        } else {
            AcceptTermsScene(model: AcceptTermsSceneViewModel(preferences: model.preferences, onNext: { navigate(to: .securityReminder) }))
        }
    }

    private var securityReminderScene: some View {
        SecurityReminderScene(
            model: SecurityReminderViewModel(
                title: Localized.Wallet.New.title,
                onNext: { navigate(to: .createWallet) },
            ),
        )
    }
}

// MARK: - Actions

extension CreateWalletNavigationStack {
    func navigate(to route: CreateWalletRoute) {
        switch route {
        case .securityReminder: navigationPath.append(Scenes.SecurityReminder())
        case .createWallet:
            do {
                try model.generateSecretPhrase()
                navigationPath.append(Scenes.CreateWallet())
            } catch {
                model.isPresentingAlertMessage = AlertMessage(title: Localized.Errors.errorOccurred, error: error)
            }
        case .verifyPhrase: navigationPath.append(Scenes.VerifyPhrase())
        }
    }

    func onVerifyPhraseComplete(words: [String]) async throws {
        try await model.createWallet(words: words)
        model.dismiss()
    }
}
