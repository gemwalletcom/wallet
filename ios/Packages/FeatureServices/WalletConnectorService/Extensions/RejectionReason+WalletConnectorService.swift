// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemServiceError
import WalletConnectSign

extension RejectionReason {
    init(from error: Error) {
        if let autoNamespacesError = error as? AutoNamespacesError {
            self = RejectionReason(from: autoNamespacesError)
            return
        }

        if case let .Status(msg) = error as? GemServiceError {
            self = switch msg {
            case "unsupported chains": .unsupportedChains
            case "wallets unsupported": .unsupportedAccounts
            default: .userRejected
            }
            return
        }
        guard let serviceError = error as? WalletConnectorServiceError else {
            self = .userRejected
            return
        }

        switch serviceError {
        case .unresolvedMethod:
            self = .unsupportedMethods
        case .unresolvedChainId:
            self = .unsupportedChains
        case .walletsUnsupported:
            self = .unsupportedAccounts
        case .wrongSignParameters, .wrongSendParameters, .invalidOrigin:
            self = .userRejected
        }
    }
}
