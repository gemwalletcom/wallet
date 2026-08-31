// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum WalletSessionServiceError: LocalizedError {
    case noWallet
    case noWalletId

    public var errorDescription: String? {
        switch self {
        case .noWallet: "No wallet found"
        case .noWalletId: "No wallet id"
        }
    }
}
