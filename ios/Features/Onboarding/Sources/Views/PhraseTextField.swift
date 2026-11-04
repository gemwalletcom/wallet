// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

@available(iOS 18.0, *)
struct PhraseTextField: View {
    let title: String
    @Binding var model: ImportInputViewModel

    @State private var selection: TextSelection?

    var body: some View {
        TextField(title, text: $model.text, selection: $selection, axis: .vertical)
            .writingToolsBehavior(.disabled)
            .onChange(of: selection) { _, selection in
                guard let offset = offset(of: selection), offset != model.cursor else { return }
                model.cursor = offset
            }
            .onChange(of: model.cursor) { _, cursor in
                guard let cursor, cursor != offset(of: selection) else { return }
                selection = TextSelection(insertionPoint: String.Index(utf16Offset: min(cursor, model.text.utf16.count), in: model.text))
            }
    }

    private func offset(of selection: TextSelection?) -> Int? {
        guard case let .selection(range) = selection?.indices, range.lowerBound <= model.text.endIndex else { return nil }
        return model.text.utf16.distance(from: model.text.startIndex, to: range.lowerBound)
    }
}
