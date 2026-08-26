// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public import enum Gemstone.GemWalletImportType

public extension KeystoreImportType {
    var walletImport: GemWalletImportType {
        switch self {
        case let .phrase(words, chains):
            .multicoinPhrase(words: words, chains: chains.map(\.rawValue))
        case let .single(words, chain):
            .singlePhrase(words: words, chain: chain.rawValue)
        case let .privateKey(text, chain):
            .privateKey(value: text, chain: chain.rawValue)
        case let .address(address, chain):
            .address(address: address, chain: chain.rawValue)
        }
    }
}
