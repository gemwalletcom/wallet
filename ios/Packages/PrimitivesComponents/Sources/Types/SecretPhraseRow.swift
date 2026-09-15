// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.secretPhraseRows

public enum SecretPhraseRow: Hashable {
    case pair(WordIndex, WordIndex)
    case single(WordIndex)
}

public extension SecretPhraseRow {
    static func rows(for words: [String]) -> [SecretPhraseRow] {
        secretPhraseRows(wordCount: UInt32(words.count)).map { $0.map(words: words) }
    }
}
