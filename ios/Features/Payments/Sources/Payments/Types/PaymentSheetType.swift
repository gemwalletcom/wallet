// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import SigningRequestService
import PrimitivesComponents

public enum PaymentSheetType: Sendable, Identifiable {
    case quotes(SheetCallback<PaymentQuotesRequest>)
    case dataCollection(SheetCallback<PaymentDataCollectionRequest>)
    case confirm(SheetCallback<SigningTransferData>)
    case signMessage(SheetCallback<SignMessagePayload>)

    public var id: String {
        callback.id
    }

    public func reject(_ error: any Error) {
        callback.reject(error)
    }

    private var callback: any SheetRejectable {
        switch self {
        case let .quotes(callback): callback
        case let .dataCollection(callback): callback
        case let .confirm(callback): callback
        case let .signMessage(callback): callback
        }
    }
}

// MARK: - SheetRejectable

extension PaymentSheetType: SheetRejectable {}
