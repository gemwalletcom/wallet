// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives

public enum PaymentSheetType: Sendable, Identifiable {
    case quotes(SigningRequestCallback<PaymentQuotesRequest>)
    case dataCollection(SigningRequestCallback<PaymentDataCollectionRequest>)

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
        }
    }
}

// MARK: - SigningRequestRejectable

extension PaymentSheetType: SigningRequestRejectable {}
