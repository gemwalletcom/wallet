// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemVerifyPhraseSession
import struct Gemstone.GemVerifyPhraseViewState
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
final class VerifyPhraseViewModel {
    private let onComplete: ([String]) async throws -> Void

    private var session: GemVerifyPhraseSession {
        didSet { viewState = session.viewState() }
    }

    private var viewState: GemVerifyPhraseViewState
    var isPresentingAlertMessage: AlertMessage?

    init(
        session: GemVerifyPhraseSession,
        onComplete: @escaping ([String]) async throws -> Void,
    ) {
        self.session = session
        viewState = session.viewState()
        self.onComplete = onComplete
    }

    var wordsIndex: Int? {
        viewState.nextIndex.map(Int.init)
    }

    var title: String {
        Localized.VerifyPhrase.title
    }

    var docsUrl: URL {
        AppUrl.docs(.howToSecureSecretPhrase)
    }

    var rows: [SecretPhraseRow] {
        SecretPhraseRow.rows(for: viewState.verified)
    }

    var groups: [[WordIndex]] {
        viewState.groups.map { group in
            group.map { WordIndex(index: Int($0.index), word: $0.word) }
        }
    }

    var buttonState: ButtonState {
        viewState.button.state
    }

    func pickWord(index: WordIndex) {
        session = session.onPick(choice: UInt32(index.index))
    }

    func isVerified(index: WordIndex) -> Bool {
        viewState.groups.joined().first { Int($0.index) == index.index }?.isPicked == true
    }
}

// MARK: - Actions

extension VerifyPhraseViewModel {
    func onContinue() {
        session = session.onCreating(isCreating: true)
        Task {
            await complete()
        }
    }

    func complete() async {
        do {
            try await onComplete(session.words)
        } catch {
            session = session.onCreating(isCreating: false)
            isPresentingAlertMessage = AlertMessage(title: Localized.Errors.createWallet(""), error: error)
        }
    }
}
