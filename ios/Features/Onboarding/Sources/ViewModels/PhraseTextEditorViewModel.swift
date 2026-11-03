// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Style
import SwiftUI

@available(iOS 26.0, *)
@Observable
@MainActor
final class PhraseTextEditorViewModel {
    let placeholder: String
    let input: ImportInputViewModel

    var content = AttributedString()
    var selection = AttributedTextSelection()

    init(placeholder: String, input: ImportInputViewModel) {
        self.placeholder = placeholder
        self.input = input
    }

    var showsPlaceholder: Bool {
        input.text.isEmpty
    }
}

// MARK: - Actions

@available(iOS 26.0, *)
extension PhraseTextEditorViewModel {
    func onChangeContent() {
        let text = String(content.characters)
        guard text != input.text else { return }
        input.text = text
    }

    func onChangeText() {
        guard String(content.characters) != input.text else { return }
        content = AttributedString(input.text)
        highlight(&content)
        selection = AttributedTextSelection(range: content.endIndex ..< content.endIndex)
    }

    func onChangeInvalidWords() {
        content.transform(updating: &selection, body: highlight)
    }

    func onChangeSelection() {
        guard let offset = cursorOffset, offset != input.cursor else { return }
        input.cursor = offset
    }

    func onChangeCursor() {
        guard let cursor = input.cursor, cursor != cursorOffset else { return }
        let index = content.utf16.index(content.startIndex, offsetBy: min(cursor, content.utf16.count))
        selection = AttributedTextSelection(range: index ..< index)
    }
}

// MARK: - Private

@available(iOS 26.0, *)
extension PhraseTextEditorViewModel {
    private var cursorOffset: Int? {
        guard case let .insertionPoint(index) = selection.indices(in: content) else { return nil }
        return content.utf16.distance(from: content.startIndex, to: index)
    }

    private func highlight(_ content: inout AttributedString) {
        let invalidWords = input.invalidWords
        content.foregroundColor = nil
        for word in content.characters.split(whereSeparator: \.isWhitespace) where invalidWords.contains(String(word).lowercased()) {
            content[word.startIndex ..< word.endIndex].foregroundColor = Colors.red
        }
    }
}
