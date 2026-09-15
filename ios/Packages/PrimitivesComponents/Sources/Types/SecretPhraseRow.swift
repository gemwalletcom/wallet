// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.secretPhraseRows

public enum SecretPhraseRow: Hashable {
    case pair(WordIndex, WordIndex)
    case single(WordIndex)
}

public extension SecretPhraseRow {
    static func rows(for words: [String]) -> [SecretPhraseRow] {
        secretPhraseRows(words: words).map { $0.map() }
    }
}
