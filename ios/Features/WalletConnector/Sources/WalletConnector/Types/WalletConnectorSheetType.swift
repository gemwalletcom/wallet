// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SigningRequestService
import WalletConnectorService

public enum WalletConnectorSheetType: Sendable, Identifiable {
    case connectionProposal(SigningRequestCallback<WCPairingProposal>)

    public var id: String {
        callback.id
    }

    public func reject(_ error: any Error) {
        callback.reject(error)
    }

    private var callback: any SigningRequestRejectable {
        switch self {
        case let .connectionProposal(callback): callback
        }
    }
}

// MARK: - SigningRequestRejectable

extension WalletConnectorSheetType: SigningRequestRejectable {}
