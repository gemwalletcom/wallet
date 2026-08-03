// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import SigningRequestService

public final class PaymentSheetPresenter: PaymentSheetPresentable, Sendable {
    public let sheets = SheetPresenter<PaymentSheetType>()

    public init() {}

    public func selectPaymentQuote(request: PaymentQuotesRequest) async throws -> String {
        try await sheets.present(payload: request, sheet: { .quotes($0) })
    }

    public func collectPaymentData(request: PaymentDataCollectionRequest) async throws -> String {
        try await sheets.present(payload: request, sheet: { .dataCollection($0) })
    }
}
