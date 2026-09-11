// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public enum ExportWalletFlow: Identifiable {
    case words([String])
    case privateKey(chain: Chain, key: String)
    case privateKeyChains(wallet: Wallet)

    public var id: String {
        switch self {
        case .words: "words"
        case .privateKey: "privateKey"
        case .privateKeyChains: "privateKeyChains"
        }
    }
}
