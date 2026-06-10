// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

internal import Gemstone

extension KeystoreImportType {
    var gemWalletImport: GemImportType? {
        switch self {
        case let .phrase(words, chains):
            .multicoinPhrase(words: words, chains: chains.map(\.rawValue))
        case let .single(words, chain):
            .singlePhrase(words: words, chain: chain.rawValue)
        case let .privateKey(text, chain):
            .privateKey(value: text, chain: chain.rawValue)
        case .address:
            nil
        }
    }
}
