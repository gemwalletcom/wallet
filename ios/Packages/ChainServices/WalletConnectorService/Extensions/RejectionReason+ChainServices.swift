// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import WalletConnectSign

extension RejectionReason {
    init(from error: Error) {
        if let autoNamespacesError = error as? AutoNamespacesError {
            self = RejectionReason(from: autoNamespacesError)
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
