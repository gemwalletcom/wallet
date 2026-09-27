// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemWalletServiceProtocol
import GemstonePrimitives
import GemstoneServices
import Primitives
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
public final class CreateWalletViewModel {
    private let service: any GemWalletServiceProtocol

    func verifyPhraseModel(onComplete: @escaping ([String]) async throws -> Void) -> VerifyPhraseSceneViewModel {
        VerifyPhraseSceneViewModel(
            words: words,
            setup: service.verifyPhraseSetup(words: words),
            onComplete: onComplete,
        )
    }

    let preferences: ObservablePreferences
    let onComplete: VoidAction

    private(set) var words: [String] = []
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

    func dismiss() {
        onComplete?()
    }
}

// MARK: - Actions

extension CreateWalletViewModel {
    func generateSecretPhrase() throws {
        words = try service.createWallet()
    }

    func createWallet(words: [String]) async throws {
        _ = try await service.importWallet(
            kind: .phrase,
            chain: .none,
            input: words.joined(separator: " "),
            nameRecord: .none,
            source: .create,
        )
    }
}
