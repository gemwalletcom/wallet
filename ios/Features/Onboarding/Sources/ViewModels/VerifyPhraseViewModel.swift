// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

@Observable
@MainActor
final class VerifyPhraseViewModel {
    private let words: [String]
    private let shuffledWords: [String]
    private let onComplete: ([String]) async throws -> Void

    var wordsVerified: [String]
    var wordsIndex: Int = 0
    var buttonState = ButtonState.disabled
    var isPresentingAlertMessage: AlertMessage?
    private var selectedIndexes = Set<WordIndex>()

    init(
        words: [String],
        shuffledWords: [String],
        onComplete: @escaping ([String]) async throws -> Void,
    ) {
        self.words = words
        self.shuffledWords = shuffledWords
        wordsVerified = Array(repeating: "", count: words.count)
        self.onComplete = onComplete
    }

    var title: String {
        Localized.VerifyPhrase.title
    }

    var docsUrl: URL {
        AppUrl.docs(.howToSecureSecretPhrase)
    }

    var rows: [SecretPhraseRow] {
        SecretPhraseRow.rows(for: wordsVerified)
    }

    var rowsSections: [[WordIndex]] {
        selectRows.chunks(4)
    }

    var selectRows: [WordIndex] {
        shuffledWords
            .enumerated()
            .map {
                WordIndex(index: $0.offset, word: $0.element)
            }
    }

    func pickWord(index: WordIndex) {
        if words[wordsIndex] == index.word {
            wordsVerified[wordsIndex] = index.word
            selectedIndexes.insert(index)
            wordsIndex += 1
        }
        // last word
        if wordsIndex == words.count {
            buttonState = .normal
        }
    }

    func isVerified(index: WordIndex) -> Bool {
        selectedIndexes.contains(index)
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
            try await onComplete(words)
        } catch {
            buttonState = .normal
            isPresentingAlertMessage = AlertMessage(title: Localized.Errors.createWallet(""), error: error)
        }
    }
}
