// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemSecretPhraseRow
import struct Gemstone.GemSecretPhraseWord

extension GemSecretPhraseRow {
    func map() -> SecretPhraseRow {
        switch self {
        case let .pair(left, right): .pair(left.map(), right.map())
        case let .single(word): .single(word.map())
        }
    }
}

extension GemSecretPhraseWord {
    func map() -> WordIndex {
        WordIndex(index: Int(index), word: word)
    }
}
