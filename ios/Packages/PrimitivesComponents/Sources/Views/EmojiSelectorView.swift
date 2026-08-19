// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct EmojiSelectorView: View {
    private let emojis: [EmojiValue]
    private let columns: Int
    private let onSelect: (EmojiValue) -> Void

    public init(
        emojis: [EmojiValue],
        columns: Int = 4,
        onSelect: @escaping (EmojiValue) -> Void,
    ) {
        self.emojis = emojis
        self.columns = columns
        self.onSelect = onSelect
    }

    public var body: some View {
        ScrollView {
            LazyVGrid(
                columns: Array(repeating: GridItem(.flexible(), spacing: .medium), count: columns),
                alignment: .center,
                spacing: .medium,
            ) {
                ForEach(emojis) { value in
                    NavigationCustomLink(
                        with: EmojiView(color: value.color, emoji: value.emoji),
                    ) {
                        onSelect(value)
                    }
                    .frame(maxWidth: .infinity)
                }
            }
            .padding(.horizontal, .medium)
        }
    }
}
