// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SigningRequestService
import PrimitivesComponents

public enum WalletConnectorSheetType: Sendable, Identifiable {
    case connectionProposal(SheetCallback<WCPairingProposal>)
    case transferData(SheetCallback<SigningTransferData>)
    case signMessage(SheetCallback<SignMessagePayload>)

    public var id: String {
        callback.id
    }

    public func reject(_ error: any Error) {
        callback.reject(error)
    }

    private var callback: any SheetRejectable {
        switch self {
        case let .connectionProposal(callback): callback
        case let .transferData(callback): callback
        case let .signMessage(callback): callback
        }
    }
}

// MARK: - SheetRejectable

extension WalletConnectorSheetType: SheetRejectable {}
