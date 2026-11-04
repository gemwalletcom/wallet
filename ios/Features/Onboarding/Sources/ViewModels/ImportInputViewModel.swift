// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemServiceError
import PrimitivesComponents
import SwiftUI

@Observable
@MainActor
final class ImportInputViewModel {
    var inputModel = InputValidationViewModel()
    var cursor: Int?

    var text: String {
        get { inputModel.text }
        set { inputModel.text = newValue }
    }

    var invalidWords: Set<String> {
        guard case let .invalidSecretPhraseWords(words) = (inputModel.error as? GemServiceError)?.text() else { return [] }
        return Set(words)
    }

    var lastWord: String {
        text.split(omittingEmptySubsequences: false, whereSeparator: \.isWhitespace).last.map(String.init) ?? ""
    }

    var isTypingLastWord: Bool {
        cursor.map { $0 >= text.utf16.count } ?? true
    }

    func update(text: String) {
        self.text = text
        cursor = nil
    }

    func update(error: (any Error)?) {
        inputModel.update(error: error)
    }

    func complete(word: String) {
        text = String(text.dropLast(lastWord.count)) + word + " "
        cursor = text.utf16.count
    }
}
