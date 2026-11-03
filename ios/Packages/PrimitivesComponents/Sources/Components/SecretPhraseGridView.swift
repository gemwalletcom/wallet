// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSecretPhraseRow
import func Gemstone.secretPhraseRows
import Style
import SwiftUI

public struct SecretPhraseGridView: View {
    private let words: [String]
    private let highlightIndex: Int?

    public init(
        words: [String],
        highlightIndex: Int? = .none,
    ) {
        self.words = words
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
                    case let .single(index):
                        HStack {
                            Spacer()
                            cell(for: index)
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

    private var rows: [GemSecretPhraseRow] {
        secretPhraseRows(wordCount: words.count)
    }

    private func cell(for index: Int) -> some View {
        HStack {
            Text("\(index + 1).")
                .fontWeight(.semibold)
                .foregroundStyle(Colors.grayLight)
                .multilineTextAlignment(.leading)
                .padding(.leading, .small)
            Text(words[index])
                .fontWeight(.semibold)
                .foregroundStyle(Colors.black)
                .allowsTightening(false)
                .multilineTextAlignment(.leading)
                .fixedSize(horizontal: false, vertical: true)
                .accessibilityIdentifier("word_\(index)")
            Spacer()
        }
        .padding(.small)
        .background(Colors.listStyleColor)
        .cornerRadius(.space10)
        .overlay {
            if highlightIndex == index {
                RoundedRectangle(cornerRadius: .space10)
                    .stroke(Colors.blue, lineWidth: 2)
            }
        }
    }
}
