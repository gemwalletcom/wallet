// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public enum WalletImportResult: Sendable {
    case new(Wallet)
    case existing(Wallet)

    public var wallet: Wallet {
        switch self {
        case let .new(wallet), let .existing(wallet):
            wallet
        }
    }
}
