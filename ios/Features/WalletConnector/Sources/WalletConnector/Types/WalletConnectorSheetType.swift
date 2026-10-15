// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemWalletConnectMessageRequest
import Primitives
import WalletConnectorService

public enum WalletConnectorSheetType: Sendable, Identifiable {
    case transferData(TransferDataCallback<WCTransferData>)
    case signMessage(TransferDataCallback<GemWalletConnectMessageRequest>)
    case connectionProposal(TransferDataCallback<WCPairingProposal>)

    public var id: AnyHashable {
        switch self {
        case let .transferData(callback): ObjectIdentifier(callback)
        case let .signMessage(callback): ObjectIdentifier(callback)
        case let .connectionProposal(callback): callback.payload.pairingId
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
