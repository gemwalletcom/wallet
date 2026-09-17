// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletImportResult
import Primitives

public extension GemWalletImportResult {
    var wallet: Primitives.Wallet {
        switch self {
        case let .new(wallet), let .existing(wallet): wallet.toPrimitives()
        }
    }
}
