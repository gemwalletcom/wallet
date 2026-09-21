// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

@available(iOS 18.0, *)
struct PhraseTextField: View {
    let title: String
    @Binding var text: String
    @Binding var cursor: Int?

    @State private var selection: TextSelection?

    var body: some View {
        TextField(title, text: $text, selection: $selection, axis: .vertical)
            .onChange(of: selection) { _, selection in
                guard let offset = offset(of: selection), offset != cursor else { return }
                cursor = offset
            }
            .onChange(of: cursor) { _, cursor in
                guard let cursor, cursor != offset(of: selection) else { return }
                selection = TextSelection(insertionPoint: String.Index(utf16Offset: min(cursor, text.utf16.count), in: text))
            }
    }

    private func offset(of selection: TextSelection?) -> Int? {
        guard case let .selection(range) = selection?.indices, range.lowerBound <= text.endIndex else { return nil }
        return text.utf16.distance(from: text.startIndex, to: range.lowerBound)
    }
}
