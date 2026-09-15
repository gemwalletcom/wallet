// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSecretPhraseRow

extension GemSecretPhraseRow {
    func map(words: [String]) -> SecretPhraseRow {
        switch self {
        case let .pair(left, right): .pair(words.wordIndex(at: left), words.wordIndex(at: right))
        case let .single(index): .single(words.wordIndex(at: index))
        }
    }
}

private extension [String] {
    func wordIndex(at index: UInt32) -> WordIndex {
        WordIndex(index: Int(index), word: self[Int(index)])
    }
}
