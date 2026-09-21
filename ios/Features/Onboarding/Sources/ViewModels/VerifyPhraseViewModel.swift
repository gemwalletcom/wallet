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

    private var session: GemVerifyPhraseSession
    var buttonState = ButtonState.disabled
    var isPresentingAlertMessage: AlertMessage?

    init(
        session: GemVerifyPhraseSession,
        onComplete: @escaping ([String]) async throws -> Void,
    ) {
        self.session = session
        self.onComplete = onComplete
    }

    private var viewState: GemVerifyPhraseViewState {
        session.viewState()
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

    var rowsSections: [[WordIndex]] {
        selectRows.chunks(4)
    }

    var selectRows: [WordIndex] {
        viewState.choices
            .enumerated()
            .map {
                WordIndex(index: $0.offset, word: $0.element.word)
            }
    }

    func pickWord(index: WordIndex) {
        session = session.onPick(choice: UInt32(index.index))
        if viewState.isComplete {
            buttonState = .normal
        }
    }

    func isVerified(index: WordIndex) -> Bool {
        viewState.choices[index.index].isPicked
    }
}

// MARK: - Actions

extension VerifyPhraseViewModel {
    func onContinue() {
        buttonState = .loading(showProgress: true)
        Task {
            await complete()
        }
    }

    func complete() async {
        do {
            try await onComplete(session.words)
        } catch {
            buttonState = .normal
            isPresentingAlertMessage = AlertMessage(title: Localized.Errors.createWallet(""), error: error)
        }
    }
}
