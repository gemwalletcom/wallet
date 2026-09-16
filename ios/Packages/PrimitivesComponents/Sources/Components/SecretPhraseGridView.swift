// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct SecretPhraseGridView: View {
    private let rows: [SecretPhraseRow]
    private let highlightIndex: Int?

    public init(
        rows: [SecretPhraseRow],
        highlightIndex: Int? = .none,
    ) {
        self.rows = rows
        self.highlightIndex = highlightIndex
    }

    public var body: some View {
        VStack(spacing: .small) {
            ForEach(rows, id: \.self) { row in
                Group {
                    switch row {
                    case let .pair(left, right):
                        HStack(spacing: .small) {
                            cell(for: left)
                            cell(for: right)
                        }
                    case let .single(word):
                        HStack {
                            Spacer()
                            cell(for: word)
                                .fixedSize(horizontal: true, vertical: false)
                            Spacer()
                        }
                    }
                }
                .padding(.vertical, .space2)
            }
        }
        .padding(.horizontal, .medium)
        .frame(maxWidth: .scene.content.maxWidth)
    }

    private func cell(for word: WordIndex) -> some View {
        HStack {
            Text("\(word.index + 1).")
                .fontWeight(.semibold)
                .foregroundStyle(Colors.grayLight)
                .multilineTextAlignment(.leading)
                .padding(.leading, .small)
            Text(word.word)
                .fontWeight(.semibold)
                .foregroundStyle(Colors.black)
                .allowsTightening(false)
                .multilineTextAlignment(.leading)
                .fixedSize(horizontal: false, vertical: true)
                .accessibilityIdentifier("word_\(word.index)")
            Spacer()
        }
        .padding(.small)
        .background(Colors.listStyleColor)
        .cornerRadius(.space10)
        .overlay {
            if highlightIndex == word.index {
                RoundedRectangle(cornerRadius: .space10)
                    .stroke(Colors.blue, lineWidth: 2)
            }
        }
    }
}

