// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum WalletSessionServiceError: LocalizedError {
    case noWalletId

    public var errorDescription: String? {
        switch self {
        case .noWalletId: "No wallet id"
        }
    }
}
