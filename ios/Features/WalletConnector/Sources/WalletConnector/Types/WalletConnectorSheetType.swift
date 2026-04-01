// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import WalletConnectorService

public enum WalletConnectorSheetType: Sendable, Identifiable {
    case transferData(TransferDataCallback<WCTransferData>)
    case signMessage(TransferDataCallback<SignMessagePayload>)
    case connectionProposal(TransferDataCallback<WCPairingProposal>)

    public var id: Int {
        switch self {
        case let .transferData(callback): callback.id.hashValue
        case let .signMessage(callback): callback.id.hashValue
        case let .connectionProposal(callback): callback.id.hashValue
        }
    }

    public func reject(_ error: Error) {
        switch self {
        case let .transferData(callback):
            callback.delegate(.failure(error))
        case let .signMessage(callback):
            callback.delegate(.failure(error))
        case let .connectionProposal(callback):
            callback.delegate(.failure(error))
        }
    }
}
