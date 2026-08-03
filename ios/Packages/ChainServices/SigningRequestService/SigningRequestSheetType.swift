// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum SigningRequestSheetType: Sendable, Identifiable {
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
        case let .transferData(callback): callback
        case let .signMessage(callback): callback
        }
    }
}

// MARK: - SigningRequestRejectable

extension SigningRequestSheetType: SigningRequestRejectable {}
