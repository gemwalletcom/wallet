// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives
import SigningRequestService

public enum PaymentSheetType: Sendable, Identifiable {
    case quotes(SigningRequestCallback<PaymentQuotesRequest>)
    case dataCollection(SigningRequestCallback<PaymentDataCollectionRequest>)
    case confirm(SigningRequestCallback<SigningTransferData>)
    case signMessage(SigningRequestCallback<SignMessagePayload>)

    public var id: String {
        callback.id
    }

    public func reject(_ error: any Error) {
        callback.reject(error)
    }

    private var callback: any SigningRequestRejectable {
        switch self {
        case let .quotes(callback): callback
        case let .dataCollection(callback): callback
        case let .confirm(callback): callback
        case let .signMessage(callback): callback
        }
    }
}

// MARK: - SigningRequestRejectable

extension PaymentSheetType: SigningRequestRejectable {}
