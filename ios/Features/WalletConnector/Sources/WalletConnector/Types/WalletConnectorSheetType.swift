// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SigningRequestService
import WalletConnectorService

public enum WalletConnectorSheetType: Sendable, Identifiable {
    case connectionProposal(SigningRequestCallback<WCPairingProposal>)
    case transferData(SigningRequestCallback<SigningTransferData>)
    case signMessage(SigningRequestCallback<SignMessagePayload>)

    public var id: String {
        callback.id
    }

    public func reject(_ error: any Error) {
        callback.reject(error)
    }

    private var callback: any SigningRequestRejectable {
        switch self {
        case let .connectionProposal(callback): callback
        case let .transferData(callback): callback
        case let .signMessage(callback): callback
        }
    }
}

// MARK: - SigningRequestRejectable

extension WalletConnectorSheetType: SigningRequestRejectable {}
