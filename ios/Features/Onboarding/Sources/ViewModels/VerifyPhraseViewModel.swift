// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemVerifyPhraseSession
import struct Gemstone.GemVerifyPhraseSetup
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
    private let words: [String]
    private let choices: [String]

    private var session: GemVerifyPhraseSession {
        didSet { viewState = session.viewState() }
    }

    private var viewState: GemVerifyPhraseViewState
    var isPresentingAlertMessage: AlertMessage?

    init(
        words: [String],
        setup: GemVerifyPhraseSetup,
        onComplete: @escaping ([String]) async throws -> Void,
    ) {
        self.words = words
        choices = setup.choices
        session = setup.session
        viewState = setup.session.viewState()
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
        SecretPhraseRow.rows(for: words.enumerated().map { $0.offset < Int(viewState.verifiedCount) ? $0.element : .empty })
    }

    var groups: [[WordIndex]] {
        viewState.groups.map { group in
            group.map { WordIndex(index: Int($0.index), word: choices[Int($0.index)]) }
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
            try await onComplete(words)
        } catch {
            session = session.onCreating(isCreating: false)
            isPresentingAlertMessage = AlertMessage(title: Localized.Errors.createWallet(""), error: error)
        }
    }
}
