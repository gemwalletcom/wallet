// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletSecret

extension GemWalletSecret: @retroactive Identifiable {
    public var id: String {
        switch self {
        case .words: "words"
        case .privateKey: "privateKey"
        }
    }
}
