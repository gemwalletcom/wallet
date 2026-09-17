// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemWalletConnectError
import enum Gemstone.GemWalletConnectRejectionReason
import WalletConnectSign

extension GemWalletConnectRejectionReason {
    init(from error: Error) {
        if let autoNamespacesError = error as? AutoNamespacesError {
            self = switch autoNamespacesError {
            case .requiredChainsNotSatisfied: .unsupportedChains
            case .requiredAccountsNotSatisfied, .emptySessionNamespacesForbidden: .unsupportedAccounts
            case .requiredMethodsNotSatisfied: .unsupportedMethods
            case .requiredEventsNotSatisfied: .unsupportedEvents
            }
            return
        }

        guard let walletConnectError = error as? GemWalletConnectError else {
            self = .userRejected
            return
        }
        self = switch walletConnectError {
        case .UnsupportedChains: .unsupportedChains
        case .UnsupportedWallets: .unsupportedAccounts
        case .InvalidOrigin, .Service: .userRejected
        }
    }
}

extension RejectionReason {
    init(_ reason: GemWalletConnectRejectionReason) {
        self = switch reason {
        case .userRejected: .userRejected
        case .unsupportedChains: .unsupportedChains
        case .unsupportedMethods: .unsupportedMethods
        case .unsupportedAccounts: .unsupportedAccounts
        case .unsupportedEvents: .unsupportedEvents
        }
    }
}
