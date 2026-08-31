// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum KeystoreImportType: Sendable {
    case phrase(words: [String], chains: [Chain])
    case single(words: [String], chain: Chain)
    case privateKey(text: String, chain: Chain)
    case address(address: String, chain: Chain)

    public var walletType: WalletType {
        switch self {
        case .phrase: .multicoin
        case .single: .single
        case .privateKey: .privateKey
        case .address: .view
        }
    }
}
